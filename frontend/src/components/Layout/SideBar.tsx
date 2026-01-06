interface SideBarProps {
    authenticated?: boolean;
    onLogin?: () => void;
    onCreatePost?: () => void;
    postSummary?: string;
}

export function SideBar({ authenticated, onLogin, onCreatePost, postSummary }: SideBarProps) {
    return (
        <aside className="w-full lg:w-72 space-y-8">
            <div className="rounded-2xl bg-white p-6 border border-slate-200 shadow-sm">
                <h3 className="font-bold text-slate-900 mb-4">About the Author</h3>
                <p className="text-sm text-slate-500 leading-relaxed">
                    A full-stack developer who loves Rust and React. Building the <strong>RustPress</strong> blog system.
                </p>
                {!authenticated && onLogin && (
                    <div className="mt-6 pt-6 border-t border-slate-100">
                        <button
                            onClick={onLogin}
                            className="w-full bg-orange-500 text-white py-2 rounded-xl text-sm font-bold hover:bg-orange-600 transition-colors"
                        >
                            Login to Publish Posts
                        </button>
                    </div>
                )}
                {authenticated && onCreatePost && (
                    <div className="mt-6 pt-6 border-t border-slate-100">
                        <button
                            onClick={onCreatePost}
                            className="w-full bg-slate-900 text-white py-2 rounded-xl text-sm font-bold hover:bg-orange-600 transition-colors"
                        >
                            Publish Post
                        </button>
                    </div>
                )}
            </div>

            {postSummary && (
                <div className="rounded-2xl bg-white p-6 border border-slate-200 shadow-sm">
                    <h3 className="font-bold text-slate-900 mb-4">About this Post</h3>
                    <p className="text-sm text-slate-600 leading-relaxed">
                        {postSummary}
                    </p>
                </div>
            )}
        </aside>
    );
}