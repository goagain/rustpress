import { useNavigate } from 'react-router-dom';
import { CreatePost } from '../Blog/CreatePost';

export function CreatePostPage() {
  const navigate = useNavigate();

  const handleSuccess = () => {
    navigate('/');
  };

  const handleCancel = () => {
    navigate('/');
  };

  return (
    <CreatePost
      onSuccess={handleSuccess}
      onCancel={handleCancel}
    />
  );
}