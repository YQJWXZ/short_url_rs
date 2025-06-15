export interface ShortUrl {
  id: number;
  long_url: string;
  short_code: string;
  short_url: string;
  created_at: string;
  expires_at?: string;
}

export interface CreateShortUrlRequest {
  long_url: string;
  custom_code?: string;
  timeout?: number;
  user_id: string;
}

export interface ApiResponse<T> {
  success: boolean;
  message: string;
  data?: T;
}
