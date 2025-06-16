import { ShortUrl, CreateShortUrlRequest, ApiResponse, QRCodeResponse } from '../types';

const API_BASE_URL = 'http://localhost:8080/api';

export const api = {
  async createShortUrl(request: CreateShortUrlRequest): Promise<ShortUrl> {
    const response = await fetch(`${API_BASE_URL}/shorten`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });

    const result: ApiResponse<ShortUrl> = await response.json();

    if (!result.success) {
      throw new Error(result.message);
    }

    return result.data!;
  },

  async getUserUrls(userId: string): Promise<ShortUrl[]> {
    const response = await fetch(`${API_BASE_URL}/urls/${userId}`);
    const result: ApiResponse<ShortUrl[]> = await response.json();

    if (!result.success) {
      throw new Error(result.message);
    }

    return result.data!;
  },

  async deleteShortUrl(id: number, userId: string): Promise<void> {
    const response = await fetch(`${API_BASE_URL}/urls/${id}/${userId}`, {
      method: 'DELETE',
    });

    const result: ApiResponse<void> = await response.json();

    if (!result.success) {
      throw new Error(result.message);
    }
  },

  async getQRCode(shortCode: string): Promise<string> {
    return `${API_BASE_URL}/qrcode/${shortCode}`;
  },
};
