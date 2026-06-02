export type Meta = {
  path: string;
  file_size: number;
  file_size_human: string;
  num_lines: number;
  num_lines_human: string;
};

export type RangeRow = {
  idx: number;
  value: unknown;
};

export type RangeResponse = {
  start: number;
  limit: number;
  rows: RangeRow[];
};

export type PreviewRow = {
  idx: number;
  byte_len: number;
  preview: string;
};

export type PreviewResponse = {
  start: number;
  limit: number;
  rows: PreviewRow[];
};

export class ApiError extends Error {
  status: number;

  constructor(message: string, status: number) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

async function fetchJson<T>(url: string): Promise<T> {
  const response = await fetch(url);
  const text = await response.text();

  if (!response.ok) {
    throw new ApiError(text || response.statusText, response.status);
  }

  return JSON.parse(text) as T;
}

export function getMeta() {
  return fetchJson<Meta>("/api/meta");
}

export function getItem(idx: number) {
  return fetchJson<unknown>(`/api/item/${idx}`);
}

export function getRange(start: number, limit: number) {
  const params = new URLSearchParams({
    start: String(start),
    limit: String(limit),
  });

  return fetchJson<RangeResponse>(`/api/range?${params}`);
}

export function getRangePreview(start: number, limit: number) {
  const params = new URLSearchParams({
    start: String(start),
    limit: String(limit),
    max_bytes: "256",
  });

  return fetchJson<PreviewResponse>(`/api/range-preview?${params}`);
}
