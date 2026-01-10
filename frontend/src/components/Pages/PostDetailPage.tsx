import { useParams, useNavigate } from 'react-router-dom';
import { PostDetail } from '../Blog/PostDetail';
import type { PostResponse } from '../../types';

interface PostDetailPageProps {
  onEdit: (post: PostResponse) => void;
}

export function PostDetailPage({ onEdit }: PostDetailPageProps) {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  if (!id) {
    return <div>Post ID is required</div>;
  }

  const handleBack = () => {
    navigate('/');
  };

  const handleEdit = (post: PostResponse) => {
    onEdit(post);
  };

  return (
    <PostDetail
      postId={id}
      onBack={handleBack}
      onEdit={handleEdit}
    />
  );
}