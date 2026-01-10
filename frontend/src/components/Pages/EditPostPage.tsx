import { useParams, useNavigate } from 'react-router-dom';
import { CreatePost } from '../Blog/CreatePost';
import type { PostResponse } from '../../types';

interface EditPostPageProps {
  editingPost: PostResponse | null;
}

export function EditPostPage({ editingPost }: EditPostPageProps) {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  if (!id || !editingPost) {
    return <div>Post ID and editing post data are required</div>;
  }

  const handleSuccess = () => {
    navigate('/');
  };

  const handleCancel = () => {
    navigate('/');
  };

  return (
    <CreatePost
      postId={editingPost.id}
      initialPost={editingPost}
      onSuccess={handleSuccess}
      onCancel={handleCancel}
    />
  );
}