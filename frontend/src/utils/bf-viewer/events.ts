import type { RecordResponse } from "@/utils/bf-viewer/api";

export const RECORD_LOADED = "record-loaded";

declare global {
  interface DocumentEventMap {
    [RECORD_LOADED]: CustomEvent<RecordResponse>;
  }
}
