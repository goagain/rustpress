import { PostList } from '../Blog/PostList';
import type { PostResponse } from '../../types';

interface HomePageProps {
  onPostSelect: (post: PostResponse) => void;
}

export function HomePage({ onPostSelect }: HomePageProps) {
  return (
    <div>
      <header className="mb-8">
        <h1 className="text-4xl font-extrabold text-slate-900 leading-tight mb-4">
          Welcome to RustPress
        </h1>
        <p className="text-slate-600">Browse our posts</p>
      </header>
      <PostList onPostSelect={onPostSelect} />
    </div>
  );
}