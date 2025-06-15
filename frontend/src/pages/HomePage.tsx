import React, { useState } from 'react';
import { api } from '../api';
import { ShortUrl } from '../types';

interface HomePageProps {
  userId: string;
}

const HomePage: React.FC<HomePageProps> = ({ userId }) => {
  const [longUrl, setLongUrl] = useState('');
  const [customCode, setCustomCode] = useState('');
  const [timeout, setTimeout] = useState('');
  const [result, setResult] = useState<ShortUrl | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError('');
    setResult(null);

    try {
      const request = {
        long_url: longUrl,
        custom_code: customCode || undefined,
        timeout: timeout ? parseInt(timeout) : undefined,
        user_id: userId,
      };

      const shortUrl = await api.createShortUrl(request);
      setResult(shortUrl);
      setLongUrl('');
      setCustomCode('');
      setTimeout('');
    } catch (err) {
      setError(err instanceof Error ? err.message : '创建短链接失败');
    } finally {
      setLoading(false);
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text).then(() => {
      alert('已复制到剪贴板！');
    });
  };

  return (
    <div className="home-page">
      <div className="container">
        <h2>生成短链接</h2>

        <form onSubmit={handleSubmit} className="url-form">
          <div className="form-group">
            <label htmlFor="longUrl">原始链接 *</label>
            <input
              type="url"
              id="longUrl"
              value={longUrl}
              onChange={(e) => setLongUrl(e.target.value)}
              placeholder="请输入要缩短的链接，如：https://example.com"
              required
            />
          </div>

          <div className="form-group">
            <label htmlFor="customCode">自定义短码（可选）</label>
            <input
              type="text"
              id="customCode"
              value={customCode}
              onChange={(e) => setCustomCode(e.target.value)}
              placeholder="自定义短码，如：mycode"
            />
          </div>

          <div className="form-group">
            <label htmlFor="timeout">有效期（秒，可选）</label>
            <input
              type="number"
              id="timeout"
              value={timeout}
              onChange={(e) => setTimeout(e.target.value)}
              placeholder="链接有效期，如：3600（1小时）"
              min="1"
            />
          </div>

          <button type="submit" disabled={loading} className="submit-button">
            {loading ? '生成中...' : '生成短链接'}
          </button>
        </form>

        {error && (
          <div className="error-message">
            {error}
          </div>
        )}

        {result && (
          <div className="result-card">
            <h3>短链接生成成功！</h3>
            <div className="result-item">
              <label>原始链接：</label>
              <span>{result.long_url}</span>
            </div>
            <div className="result-item">
              <label>短链接：</label>
              <div className="url-with-copy">
                <span className="short-url">{result.short_url}</span>
                <button
                  type="button"
                  onClick={() => copyToClipboard(result.short_url)}
                  className="copy-button"
                >
                  复制
                </button>
              </div>
            </div>
            <div className="result-item">
              <label>创建时间：</label>
              <span>{new Date(result.created_at).toLocaleString()}</span>
            </div>
            {result.expires_at && (
              <div className="result-item">
                <label>过期时间：</label>
                <span>{new Date(result.expires_at).toLocaleString()}</span>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default HomePage;
