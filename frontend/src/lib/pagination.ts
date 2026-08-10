// Pagination mirror of the backend `PageQuery`/`PaginatedResponse<T>` shape:
//   GET ?page=&page_size= → { items, total, page, page_size }

export interface PageParams {
  page: number;
  pageSize: number;
}

export const DEFAULT_PAGE_SIZE = 20;
export const MAX_PAGE_SIZE = 100; // backend clamps to [1, 100]

export function defaultPage(): PageParams {
  return { page: 1, pageSize: DEFAULT_PAGE_SIZE };
}

export function sanitizePage(p: PageParams): PageParams {
  return {
    page: Math.max(1, Math.floor(p.page)),
    pageSize: Math.min(MAX_PAGE_SIZE, Math.max(1, Math.floor(p.pageSize))),
  };
}

export function pageToQueryString(p: PageParams): string {
  const s = sanitizePage(p);
  return `?page=${s.page}&page_size=${s.pageSize}`;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
}

export function totalPages(p: PaginatedResponse<unknown>): number {
  if (p.total <= 0) return 1;
  return Math.ceil(p.total / p.page_size);
}
