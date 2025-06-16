import React from 'react';
import { QRCodeSVG } from 'qrcode.react';

interface QRCodeProps {
  longUrl: string;
}

const QRCode: React.FC<QRCodeProps> = ({ longUrl }) => {
  return (
    <div className="qr-code-container">
      <QRCodeSVG value={longUrl} size={200} />
      <div className="qr-code-tip">扫描二维码</div>
    </div>
  );
};

export default QRCode;
