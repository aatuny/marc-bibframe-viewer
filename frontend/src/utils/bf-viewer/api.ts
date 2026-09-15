import type { Triple } from "./bibframe";

export interface RecordResponse {
  key: string;
  record_index: number;
  has_next: boolean;
  marc: string;
  bibframe_xml: string;
  turtle: string;
  triples: Triple[];
}

export interface RecordQuery {
  record_index?: number;
  key?: string;
}

export async function fetchRecord(q: RecordQuery): Promise<RecordResponse> {
  let url = "/api/record";

  if (q.record_index) {
    const query = new URLSearchParams({ record_index: String(q.record_index) });
    url += `?${query}`;
  } else if (q.key) {
    const query = new URLSearchParams({ key: q.key });
    url += `?${query}`;
  }

  const response = await fetch(url);

  if (!response.ok) {
    throw new Error(`API responded with HTTP status ${response.status}`);
  }

  return response.json();
}
