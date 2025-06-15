import React, { useState } from 'react';
import HomePage from './pages/HomePage';
import ManagePage from './pages/ManagePage';

const App: React.FC = () => {
  const [currentPage, setCurrentPage] = useState<'home' | 'manage'>('home');
  const [userId] = useState<string>(() => {
    // Simple user ID generation for demo purposes
    return localStorage.getItem('userId') || (() => {
      const id = 'user_' + Math.random().toString(36).substr(2, 9);
      localStorage.setItem('userId', id);
      return id;
    })();
  });

  return (
    <div className="app">
      <nav className="navbar">
        <div className="nav-container">
          <h1 className="nav-title">短链接服务</h1>
          <div className="nav-buttons">
            <button
              className={`nav-button ${currentPage === 'home' ? 'active' : ''}`}
              onClick={() => setCurrentPage('home')}
            >
              生成短链接
            </button>
            <button
              className={`nav-button ${currentPage === 'manage' ? 'active' : ''}`}
              onClick={() => setCurrentPage('manage')}
            >
              管理链接
            </button>
          </div>
        </div>
      </nav>

      <main className="main-content">
        {currentPage === 'home' ? (
          <HomePage userId={userId} />
        ) : (
          <ManagePage userId={userId} />
        )}
      </main>
    </div>
  );
};

export default App;
