export const RDF_TYPE = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
export const ROOT_DISPLAY_TYPES = ["Work", "Instance", "Item"]; // Note: order here is also display order

export interface Triple {
  subject: string;
  subject_kind: string;
  predicate: string;
  object: string;
  object_kind: string;
  datatype?: string;
  language?: string;
}

export type TripleIndex = Map<string, Triple[]>;

export function shortenIri(iri: string): string {
  const lastSeparatorIdx = Math.max(iri.lastIndexOf("#"), iri.lastIndexOf("/"));

  // If no separator is observed, IRI cannot be shortened
  if (lastSeparatorIdx === -1) {
    return iri;
  }

  // Shortened version is just the last definition after last separator
  return iri.slice(lastSeparatorIdx + 1);
}

export function humanize(name: string): string {
  // Add spacing: mainTitle -> main Title
  const camelCaseSeam = /([a-z0-9])([A-Z])/g;
  const spaced = name.replace(camelCaseSeam, "$1 $2");

  // Uppercase first letter
  const firstCharacter = spaced.charAt(0).toUpperCase();

  return `${firstCharacter}${spaced.slice(1)}`;
}

export function indexTriples(triples: Triple[]): TripleIndex {
  const bySubject = new Map();

  for (const triple of triples) {
    const hasSubject = bySubject.has(triple.subject);
    if (!hasSubject) {
      bySubject.set(triple.subject, []);
    }

    const subject = bySubject.get(triple.subject);
    subject.push(triple);
  }

  return bySubject;
}

export function getSubjectType(
  bySubject: TripleIndex,
  subject: string,
): string | null {
  const subjectTriples = bySubject.get(subject) || [];
  const typeTriple = subjectTriples.find((t) => t.predicate === RDF_TYPE);

  if (!typeTriple) {
    return null;
  }

  return shortenIri(typeTriple.object);
}

export function isLeaf(t: Triple, bySubject: TripleIndex): boolean {
  // All literals are leafs always
  if (t.object_kind === "literal") {
    return true;
  }

  const nested = bySubject.get(t.object) || [];

  // Note: returns true on empty arrays
  return nested.every((t) => t.predicate === RDF_TYPE);
}

export function getRootDisplaySubjects(bySubject: TripleIndex) {
  const subjects = [...bySubject.keys()];
  const rootSubjects = subjects.filter((subject) => {
    const subjectType = getSubjectType(bySubject, subject);
    return subjectType && ROOT_DISPLAY_TYPES.includes(subjectType);
  });

  return rootSubjects.sort((a, b) => {
    const subjectTypeA = getSubjectType(bySubject, a) || "Unknown";
    const subjectTypeB = getSubjectType(bySubject, b) || "Unknown";

    const rankA = ROOT_DISPLAY_TYPES.indexOf(subjectTypeA);
    const rankB = ROOT_DISPLAY_TYPES.indexOf(subjectTypeB);
    return rankA - rankB;
  });
}
