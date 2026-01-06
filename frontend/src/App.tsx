import { useState, useEffect } from 'react';
import { NavBar } from './components/Layout/NavBar';
import { SideBar } from './components/Layout/SideBar';
import { LoginModal } from './components/Auth/LoginModal';
import { PostList } from './components/Blog/PostList';
import { PostDetail } from './components/Blog/PostDetail';
import { CreatePost } from './components/Blog/CreatePost';
import { PostVersions } from './components/Blog/PostVersions';
import { isAuthenticated, clearTokens } from './services/api';
import type { PostResponse } from './types';

export default function App() {
  const [authenticated, setAuthenticated] = useState(isAuthenticated());
  const [showLoginModal, setShowLoginModal] = useState(false);
  const [selectedPost, setSelectedPost] = useState<PostResponse | null>(null);
  const [editingPost, setEditingPost] = useState<PostResponse | null>(null);
  const [showVersions, setShowVersions] = useState(false);
  const [currentView, setCurrentView] = useState<'list' | 'detail' | 'create' | 'edit'>('list');

  // Monitor authentication state changes
  useEffect(() => {
    const checkAuth = () => {
      setAuthenticated(isAuthenticated());
    };
    
    // Periodically check authentication state (handle token expiration, etc.)
    const interval = setInterval(checkAuth, 1000);
    return () => clearInterval(interval);
  }, []);

  // Check URL parameters for post ID
  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search);
    const postId = urlParams.get('post');
    if (postId) {
      setCurrentView('detail');
    }
  }, []);

  const handleLoginSuccess = () => {
    setAuthenticated(true);
    setShowLoginModal(false);
  };

  const handleLogout = () => {
    clearTokens();
    setAuthenticated(false);
    setSelectedPost(null);
    setCurrentView('list');
  };

  const handlePostSelect = (post: PostResponse) => {
    setSelectedPost(post);
    setCurrentView('detail');
    window.history.pushState({}, '', `?post=${post.id}`);
  };

  const handleBackToList = () => {
    setSelectedPost(null);
    setCurrentView('list');
    window.history.pushState({}, '', '/');
  };

  const handleCreatePost = () => {
    setCurrentView('create');
  };

  const handlePostCreated = () => {
    setCurrentView('list');
    // Optionally refresh the post list
    window.location.reload();
  };

  const handleCancelCreate = () => {
    setCurrentView('list');
    setEditingPost(null);
  };

  const handleEditPost = (post: PostResponse) => {
    setEditingPost(post);
    setCurrentView('edit');
  };

  const handlePostUpdated = () => {
    setCurrentView('list');
    setEditingPost(null);
    setSelectedPost(null);
    window.location.reload();
  };

  const handleVersionsClose = () => {
    setShowVersions(false);
  };

  const handleVersionRestored = () => {
    setShowVersions(false);
    if (selectedPost) {
      // Reload the post
      window.location.reload();
    }
  };

  return (
    <div className="min-h-screen bg-slate-50 font-sans selection:bg-orange-100">
      <NavBar 
        authenticated={authenticated}
        onLogin={() => setShowLoginModal(true)}
        onLogout={handleLogout} 
      />

      <main className="mx-auto max-w-5xl px-4 py-12 sm:px-6 lg:px-8">
        <div className="flex flex-col lg:flex-row gap-12">
          {/* Post content area - viewable without login */}
          <div className="flex-1 min-w-0">
            {currentView === 'create' ? (
              <CreatePost onSuccess={handlePostCreated} onCancel={handleCancelCreate} />
            ) : currentView === 'edit' && editingPost ? (
              <CreatePost 
                postId={editingPost.id}
                initialPost={editingPost}
                onSuccess={handlePostUpdated} 
                onCancel={handleCancelCreate} 
              />
            ) : currentView === 'detail' && selectedPost ? (
              <PostDetail 
                postId={selectedPost.id} 
                onBack={handleBackToList}
                onEdit={handleEditPost}
              />
            ) : (
              <div>
                <header className="mb-8">
                  <h1 className="text-4xl font-extrabold text-slate-900 leading-tight mb-4">
                    Welcome to RustPress
                  </h1>
                  <p className="text-slate-600">Browse our posts</p>
                </header>
                <PostList onPostSelect={handlePostSelect} />
              </div>
            )}
          </div>

          <SideBar
            authenticated={authenticated}
            onLogin={() => setShowLoginModal(true)}
            onCreatePost={handleCreatePost}
            postSummary={selectedPost?.description || undefined}
          />
        </div>
      </main>

      {/* Login modal */}
      <LoginModal 
        isOpen={showLoginModal} 
        onClose={() => setShowLoginModal(false)}
        onSuccess={handleLoginSuccess}
      />

      {/* Version history modal */}
      {showVersions && selectedPost && (
        <PostVersions
          postId={selectedPost.id}
          onClose={handleVersionsClose}
          onRestore={handleVersionRestored}
        />
      )}
    </div>
  );
}