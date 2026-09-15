mod error;
pub use error::Error;

use oxrdf::{NamedOrBlankNode, Term};
use oxrdfio::{RdfFormat, RdfParser, RdfSerializer};

use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::FileExt;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use quick_xml::{Reader, events::Event};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

const XSLT_TIMEOUT: Duration = Duration::from_secs(4);

pub struct MarcIndexEntry {
    pub key: String,
    pub start: u64,
    pub length: u64,
}

#[derive(serde::Serialize)]
pub struct Triple {
    pub subject: String,
    pub subject_kind: String,
    pub predicate: String,
    pub object: String,
    pub object_kind: String,
    pub datatype: Option<String>,
    pub language: Option<String>,
}

#[derive(Clone, Copy)]
enum Field {
    F001,
    F003,
}

/// Scan MARCXML in order to create a index containing byte offset for every marc record. This is the magic that allows O(1) finds during API calls.
///
/// The key for index is `(<003>)<001>`. Records missing either of the required fields are skipped.
///
/// This index needs to be created every time the source XML file has changed. Note that this includes any changes in whitespace also.
pub fn create_index(marc_xml_path: &Path) -> Result<Vec<MarcIndexEntry>, Error> {
    let mut reader = Reader::from_file(marc_xml_path)?;

    let mut buf = Vec::new();
    let mut records: Vec<MarcIndexEntry> = Vec::new();

    let mut start: Option<u64> = None;

    // Use combination of f003+f001 for unique record numbers
    let mut current_field: Option<Field> = None;

    let mut control_number: Option<String> = None;
    let mut org_code: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) if e.local_name().as_ref() == "record" => {
                // Start tag length is missing from reader.buffer_position()
                let after_tag = reader.buffer_position();
                start = Some(after_tag - (e.len() as u64 + 2)); // the +2 is < and >
            }
            Event::Start(e) if e.local_name().as_ref() == "controlfield" => {
                for attr in e.attributes() {
                    let attr = attr?;
                    if attr.key.local_name().as_ref() == "tag" {
                        current_field = match attr.value.as_ref() {
                            "001" => Some(Field::F001),
                            "003" => Some(Field::F003),
                            _ => None,
                        };
                    }
                }
            }
            Event::End(e) if e.local_name().as_ref() == "controlfield" => {
                current_field = None;
            }
            Event::Text(e) if current_field.is_some() => {
                let v = e.xml10_content().into_owned();

                match current_field {
                    Some(Field::F001) => control_number = Some(v),
                    Some(Field::F003) => org_code = Some(v),
                    _ => {}
                }
            }
            Event::End(e) if e.local_name().as_ref() == "record" => {
                if let (Some(s), Some(cn), Some(org)) =
                    (start.take(), control_number.take(), org_code.take())
                {
                    let end = reader.buffer_position();
                    records.push(MarcIndexEntry {
                        key: format!("({org}){cn}"),
                        start: s,
                        length: end - s,
                    });
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(records)
}

/// Write index file to selected path. The format for each entry is as follows:
/// <key>\t<byte_start>\t<byte_length>
pub fn write_index(records: &[MarcIndexEntry], path: &Path) -> Result<(), Error> {
    let file = File::create(path)?;
    let mut stream = BufWriter::new(file);
    for r in records {
        writeln!(stream, "{}\t{}\t{}", r.key, r.start, r.length)?;
    }

    stream.flush()?;
    Ok(())
}

/// Reads index file from given path
pub fn read_index(path: &Path) -> Result<Vec<MarcIndexEntry>, Error> {
    let mut records: Vec<MarcIndexEntry> = Vec::new();

    let file = File::open(path)?;
    let stream = BufReader::new(file);

    for line in stream.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split('\t').collect();
        let [key, start, length] = parts[..] else {
            return Err(Error::Index(format!(
                "expected 3 fields, got {}",
                parts.len()
            )));
        };

        let start: u64 = start.parse()?;
        let length: u64 = length.parse()?;

        records.push(MarcIndexEntry {
            key: key.into(),
            start,
            length,
        })
    }

    Ok(records)
}

/// Reads given entry from given index file.
///
/// Note that this reading function does not raise error if the bytes given are not correct.
/// This might happen if you have edited the source XML file and not re-generated the index file.
///
/// The function returns <marc:record> fragment which cannot be directly used in xsltproc conversion.
/// Please utilize [`read_records_as_collection`] to get documents that are directly compatible with the conversion.
fn read_record(file: &File, entry: &MarcIndexEntry) -> Result<String, Error> {
    // Allocate exact number of bytes needed for buffer
    let mut buf = vec![0u8; entry.length as usize];

    // Read exact number of bytes to fill the buffer
    file.read_exact_at(&mut buf, entry.start)?;

    // Return read bytes as string
    Ok(String::from_utf8(buf)?)
}

/// Reads given index entries from the XML file.
/// After read, wraps entries so that the output is valid MARCXML that is ready for xsltproc-based conversion.
pub fn read_records_as_collection(
    file: &File,
    entries: &[MarcIndexEntry],
) -> Result<String, Error> {
    let mut wrapped = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
    <marc:collection xmlns:marc="http://www.loc.gov/MARC21/slim">"#,
    );

    for e in entries {
        let record = read_record(file, e)?;
        wrapped.push_str(&record);
    }

    wrapped.push_str("</marc:collection>");

    Ok(wrapped)
}

/// Runs proposed preprocessing for MARC->BIBFRAME conversion to given records using xsltproc
pub async fn preprocess_collection(
    marc_collection_xml: &str,
    xsl_dir: &Path,
) -> Result<String, Error> {
    run_xsltproc(
        marc_collection_xml,
        &xsl_dir.join("ConvSpec-Preprocess0-Splitting.xsl"),
    )
    .await
}

/// Runs MARC->BIBFRAME conversion to given preprocessed records using xsltproc
pub async fn to_bibframe(preprocessed_xml: &str, xsl_dir: &Path) -> Result<String, Error> {
    let bibframe_xml = run_xsltproc(preprocessed_xml, &xsl_dir.join("marc2bibframe2.xsl")).await?;

    Ok(bibframe_xml)
}

/// Wrapper for running xsltproc subprocess
pub async fn run_xsltproc(input: &str, stylesheet_path: &Path) -> Result<String, Error> {
    let mut xsltproc_process = Command::new("xsltproc")
        .arg(stylesheet_path)
        .arg("-") // Use stdin instead of file path
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let mut stdin = xsltproc_process
        .stdin
        .take()
        .ok_or_else(|| Error::Xslt("stdin not piped".into()))?;

    stdin.write_all(input.as_bytes()).await?; // Write input to stdin for the child process

    drop(stdin); // Explicitly dispose stdin

    let output = tokio::time::timeout(XSLT_TIMEOUT, xsltproc_process.wait_with_output()).await;

    let output = match output {
        Ok(r) => r?,
        Err(_) => return Err(Error::Xslt("timeout".into())),
    };

    // Process stderr
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(Error::Xslt(stderr));
    }

    Ok(String::from_utf8(output.stdout)?)
}

/// Converts RDF XML outputted by the conversion to Turtle.
pub fn rdfxml_to_turtle(input: &str) -> Result<String, Error> {
    // for_writer binds serializer to destination
    let mut serializer = RdfSerializer::from_format(RdfFormat::Turtle)
        .with_prefix("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#")?
        .with_prefix("rdfs", "http://www.w3.org/2000/01/rdf-schema#")?
        .with_prefix("bf", "http://id.loc.gov/ontologies/bibframe/")?
        .with_prefix("bflc", "http://id.loc.gov/ontologies/bflc/")?
        .with_prefix("madsrdf", "http://www.loc.gov/mads/rdf/v1#")?
        .with_prefix("vocab", "http://id.loc.gov/vocabulary/")?
        .with_prefix("mstatus", "http://id.loc.gov/vocabulary/mstatus/")?
        .with_prefix("relators", "http://id.loc.gov/vocabulary/relators/")?
        .for_writer(Vec::new());

    // for_reader produces iterator
    // A quad is one statement: subject, predicate, object, plus an optional graph name. The unit RDF is made of.
    for quad in RdfParser::from_format(RdfFormat::RdfXml).for_reader(input.as_bytes()) {
        serializer.serialize_quad(&quad?)?;
    }

    let bytes = serializer.finish()?;

    Ok(String::from_utf8(bytes)?)
}

/// Converts RDF XML outputted by the conversion to custom triplets.
/// These triplets are utilized by the customized frontend view.
pub fn rdfxml_to_triples(input: &str) -> Result<Vec<Triple>, Error> {
    let mut triples: Vec<Triple> = Vec::new();

    for quad in RdfParser::from_format(RdfFormat::RdfXml).for_reader(input.as_bytes()) {
        let q = quad?;

        // check node type: iri or blank
        let (subject, subject_kind) = match q.subject {
            NamedOrBlankNode::NamedNode(n) => (n.into_string(), "iri".into()),
            NamedOrBlankNode::BlankNode(b) => (b.into_string(), "bnode".into()),
        };

        let (object, object_kind, datatype, language) = match q.object {
            Term::NamedNode(n) => (n.into_string(), "iri".into(), None, None),
            Term::BlankNode(b) => (b.into_string(), "bnode".into(), None, None),
            Term::Literal(l) => {
                let lang = l.language().map(|s| s.to_string());
                let dt = l.datatype().as_str().to_string();
                (l.value().to_string(), "literal".into(), Some(dt), lang)
            }
        };

        triples.push(Triple {
            subject,
            subject_kind,
            predicate: q.predicate.into_string(),
            object,
            object_kind,
            datatype,
            language,
        });
    }

    Ok(triples)
}
