export function marcToText(xml: string): string {
  // DOMParser never throws — it returns a document containing a parsererror element instead.
  const doc = new DOMParser().parseFromString(xml, "application/xml");
  if (doc.querySelector("parsererror")) {
    return xml;
  }

  // Render each field as a line for simplicity
  const lines = [];

  const leader = doc.querySelector("leader");
  if (leader && leader.textContent) {
    lines.push(formatLeader(leader.textContent));
  }

  for (const f of doc.querySelectorAll("controlfield")) {
    if (f.textContent) {
      lines.push(formatControlField(f.getAttribute("tag"), f.textContent));
    }
  }

  for (const f of doc.querySelectorAll("datafield")) {
    const ind1 = (f.getAttribute("ind1") || " ").replace(" ", "#");
    const ind2 = (f.getAttribute("ind2") || " ").replace(" ", "#");

    const subfields: Element[] = Array.from(f.querySelectorAll("subfield"));
    const formattedSubfields = subfields.map(formatSubfield);
    const formattedSubfieldsString = formattedSubfields.join(" ");

    lines.push(
      formatDataField(
        f.getAttribute("tag"),
        ind1,
        ind2,
        formattedSubfieldsString,
      ),
    );
  }

  return lines.join("\n");
}

export function formatLeader(leader: string): string {
  return `LDR    ${leader}`;
}

export function formatControlField(tag: string | null, value: string): string {
  const renderedTag = tag || "UND";
  return `${renderedTag}    ${value}`;
}

export function formatSubfield(e: Element): string {
  const code = e.getAttribute("code");
  const value = e.textContent;

  if (!code || !value) {
    return "<malformed subfield>";
  }

  return `$${code} ${value}`;
}

export function formatDataField(
  tag: string | null,
  ind1: string,
  ind2: string,
  subfieldString: string,
): string {
  const renderedTag = tag || "UND";
  return `${renderedTag} ${ind1}${ind2} ${subfieldString}`;
}
