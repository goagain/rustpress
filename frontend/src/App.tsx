import { useState, useEffect } from 'react';
import { Routes, Route, useLocation, useNavigate } from 'react-router-dom';
import { NavBar } from './components/Layout/NavBar';
import { SideBar } from './components/Layout/SideBar';
import { LoginModal } from './components/Auth/LoginModal';
import { PostVersions } from './components/Blog/PostVersions';
import { HomePage } from './components/Pages/HomePage';
import { PostDetailPage } from './components/Pages/PostDetailPage';
import { CreatePostPage } from './components/Pages/CreatePostPage';
import { EditPostPage } from './components/Pages/EditPostPage';
import { api, isAuthenticated, clearTokens } from './services/api';
import type { PostResponse } from './types';

export default function App() {
  const [authenticated, setAuthenticated] = useState(isAuthenticated());
  const [showLoginModal, setShowLoginModal] = useState(false);
  const [editingPost, setEditingPost] = useState<PostResponse | null>(null);
  const [showVersions, setShowVersions] = useState(false);
  const [currentPost, setCurrentPost] = useState<PostResponse | null>(null);
  const location = useLocation();
  const navigate = useNavigate();

  // Monitor authentication state changes
  useEffect(() => {
    const checkAuth = () => {
      setAuthenticated(isAuthenticated());
    };

    // Periodically check authentication state (handle token expiration, etc.)
    const interval = setInterval(checkAuth, 1000);
    return () => clearInterval(interval);
  }, []);

  // Load current post data when navigating to post detail page
  useEffect(() => {
    const loadCurrentPost = async () => {
      const pathParts = location.pathname.split('/');
      if (pathParts[1] === 'posts' && pathParts[2] && !pathParts[3]) {
        // We're on a post detail page
        try {
          const postId = parseInt(pathParts[2]);
          if (!isNaN(postId)) {
            const post = await api.getPost(postId);
            setCurrentPost(post);
          }
        } catch (error) {
          console.error('Failed to load post:', error);
          setCurrentPost(null);
        }
      } else {
        // Not on a post detail page, clear current post
        setCurrentPost(null);
      }
    };

    loadCurrentPost();
  }, [location.pathname]);

  const handleLoginSuccess = () => {
    setAuthenticated(true);
    setShowLoginModal(false);
  };

  const handleLogout = () => {
    clearTokens();
    setAuthenticated(false);
    setEditingPost(null);
    setCurrentPost(null);
    navigate('/');
  };

  const handlePostSelect = (post: PostResponse) => {
    navigate(`/posts/${post.id}`);
  };

  const handleCreatePost = () => {
    navigate('/posts/create');
  };

  const handleEditPost = (post: PostResponse) => {
    setEditingPost(post);
    navigate(`/posts/${post.id}/edit`);
  };

  const handleVersionsClose = () => {
    setShowVersions(false);
  };

  const handleVersionRestored = () => {
    setShowVersions(false);
    window.location.reload();
  };

  // Get current post summary for sidebar
  const getPostSummary = (): string | undefined => {
    return currentPost?.description || undefined;
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
            <Routes>
              <Route path="/" element={<HomePage onPostSelect={handlePostSelect} />} />
              <Route path="/posts/:id" element={<PostDetailPage onEdit={handleEditPost} />} />
              <Route path="/posts/create" element={<CreatePostPage />} />
              <Route path="/posts/:id/edit" element={<EditPostPage editingPost={editingPost} />} />
            </Routes>
          </div>

          <SideBar
            authenticated={authenticated}
            onLogin={() => setShowLoginModal(true)}
            onCreatePost={handleCreatePost}
            postSummary={getPostSummary()}
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
      {showVersions && (
        <PostVersions
          postId="0" // This will be set when opening versions
          onClose={handleVersionsClose}
          onRestore={handleVersionRestored}
        />
      )}
    </div>
  );
}