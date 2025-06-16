import React, { useState, useEffect } from 'react';
import { api } from '../api';
import { ShortUrl } from '../types';
import QRCode from '../components/QRCode';
import '../styles/qrcode.css';

interface ManagePageProps {
  userId: string;
}

const ManagePage: React.FC<ManagePageProps> = ({ userId }) => {
  const [urls, setUrls] = useState<ShortUrl[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [selectedUrl, setSelectedUrl] = useState<string | null>(null);

  useEffect(() => {
    loadUrls();
  }, [userId]);

  const loadUrls = async () => {
    try {
      setLoading(true);
      const userUrls = await api.getUserUrls(userId);
      setUrls(userUrls);
    } catch (err) {
      setError(err instanceof Error ? err.message : '加载链接失败');
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (id: number) => {
    if (!confirm('确定要删除这个短链接吗？')) {
      return;
    }

    try {
      await api.deleteShortUrl(id, userId);
      setUrls(urls.filter(url => url.id !== id));
    } catch (err) {
      alert(err instanceof Error ? err.message : '删除失败');
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text).then(() => {
      alert('已复制到剪贴板！');
    });
  };

  const isExpired = (expiresAt?: string) => {
    if (!expiresAt) return false;
    return new Date(expiresAt) < new Date();
  };

  if (loading) {
    return (
      <div className="manage-page">
        <div className="container">
          <div className="loading">加载中...</div>
        </div>
      </div>
    );
  }

  return (
    <div className="manage-page">
      <div className="container">
        <h2>我的短链接</h2>

        {error && (
          <div className="error-message">
            {error}
          </div>
        )}

        {urls.length === 0 ? (
          <div className="empty-state">
            <p>您还没有创建任何短链接</p>
          </div>
        ) : (
          <div className="urls-list">
            {urls.map((url) => (
              <div key={url.id} className={`url-card ${isExpired(url.expires_at) ? 'expired' : ''}`}>
                <div className="url-info">
                  <div className="url-row">
                    <label>原始链接：</label>
                    <span className="long-url" title={url.long_url}>
                      {url.long_url.length > 50
                        ? `${url.long_url.substring(0, 50)}...`
                        : url.long_url}
                    </span>
                  </div>

                  <div className="url-row">
                    <label>短链接：</label>
                    <div className="url-with-copy">
                      <span className="short-url">{url.short_url}</span>
                      <button
                        type="button"
                        onClick={() => copyToClipboard(url.short_url)}
                        className="copy-button"
                      >
                        复制
                      </button>
                    </div>
                  </div>

                  <div className="url-meta">
                    <span>创建时间：{new Date(url.created_at).toLocaleString()}</span>
                    {url.expires_at && (
                      <span className={isExpired(url.expires_at) ? 'expired-text' : ''}>
                        过期时间：{new Date(url.expires_at).toLocaleString()}
                        {isExpired(url.expires_at) && ' (已过期)'}
                      </span>
                    )}
                  </div>
                </div>

                <div className="url-actions">
                  <button
                    type="button"
                    onClick={() => setSelectedUrl(selectedUrl === url.short_code ? null : url.short_code)}
                    className="qr-button"
                  >
                    {selectedUrl === url.short_code ? '隐藏二维码' : '显示二维码'}
                  </button>
                  <button
                    type="button"
                    onClick={() => handleDelete(url.id)}
                    className="delete-button"
                  >
                    删除
                  </button>
                </div>
                {selectedUrl === url.short_code && (
                  <QRCode longUrl={url.long_url} />
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default ManagePage;
