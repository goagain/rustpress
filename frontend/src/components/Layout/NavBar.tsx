import { useState } from "react";
import { PenTool, Home, Search, Github, Menu, X, LayoutGrid, LogOut, User, LogIn } from 'lucide-react';

const NavLink = ({ icon, label, active = false, onClick }: { icon?: React.ReactNode, label: string, active?: boolean, onClick?: () => void }) => {
    return <button 
        onClick={onClick}
        className={`flex items-center gap-1.5 text-sm font-semibold transition-colors ${active ? 'text-orange-500' : 'text-slate-600 hover:text-orange-500'
        }`}>
        {icon}
        {label}
    </button>
}

interface NavBarProps {
    authenticated: boolean;
    onLogin?: () => void;
    onLogout?: () => void;
}

export function NavBar({ authenticated, onLogin, onLogout }: NavBarProps) {
    const [isMenuOpen, setIsMenuOpen] = useState(false);

    return (
        <nav className="sticky top-0 z-50 w-full border-b border-slate-200 bg-white/80 backdrop-blur-md">
            <div className="mx-auto max-w-5xl px-4 sm:px-6 lg:px-8">
                {/* Desktop Nav */}
                <div className="hidden md:flex items-center gap-8">
                    {/* Logo */}
                    <div className="flex items-center gap-2 group cursor-pointer">
                        <div className="bg-orange-500 p-1.5 rounded-lg group-hover:rotate-12 transition-transform">
                            <PenTool size={20} className="text-white" />
                        </div>
                        <span className="text-xl font-bold tracking-tight text-slate-900">
                            Rust<span className="text-orange-500">Press</span>
                        </span>
                    </div>
                    <NavLink icon={<Home size={18} />} label="Home" active />
                    <NavLink icon={<LayoutGrid size={18} />} label="Categories" />
                    <button className="p-2 text-slate-500 hover:text-orange-500 transition-colors cursor-pointer">
                        <Search size={20} />
                    </button>
                    {authenticated ? (
                        <>
                            <button className="p-2 text-slate-500 hover:text-orange-500 transition-colors cursor-pointer" title="User">
                                <User size={20} />
                            </button>
                            {onLogout && (
                                <button 
                                    onClick={onLogout}
                                    className="p-2 text-slate-500 hover:text-red-500 transition-colors cursor-pointer" 
                                    title="Logout"
                                >
                                    <LogOut size={20} />
                                </button>
                            )}
                        </>
                    ) : (
                        onLogin && (
                            <button 
                                onClick={onLogin}
                                className="flex items-center gap-1.5 px-3 py-1.5 text-sm font-semibold text-orange-600 hover:text-orange-700 transition-colors"
                                title="Login"
                            >
                                <LogIn size={18} />
                                Login
                            </button>
                        )
                    )}
                    <a href="https://github.com/goagain/rustpress" target="_blank" className="p-2 text-slate-500 hover:text-slate-900">
                        <Github size={20} />
                    </a>
                </div>
                {/* Mobile Menu Button */}
                <div className="md:hidden">
                    <button onClick={() => setIsMenuOpen(!isMenuOpen)} className="p-2 text-slate-600 cursor-pointer">
                        {isMenuOpen ? <X size={24} /> : <Menu size={24} />}
                    </button>
                </div>
            </div>
            {/* Mobile Dropdown */}
            {isMenuOpen && (
                <div className="md:hidden border-b border-slate-200 bg-white p-4 space-y-4 animate-in fade-in slide-in-from-top-2">
                    <NavLink label="Home" active />
                    <NavLink label="Categories" />
                    {!authenticated && onLogin && (
                        <button 
                            onClick={onLogin}
                            className="flex items-center gap-1.5 text-sm font-semibold text-orange-600 hover:text-orange-700 w-full"
                        >
                            <LogIn size={18} />
                            Login
                        </button>
                    )}
                    {authenticated && onLogout && (
                        <button 
                            onClick={onLogout}
                            className="flex items-center gap-1.5 text-sm font-semibold text-red-600 hover:text-red-700 w-full"
                        >
                            <LogOut size={18} />
                            Logout
                        </button>
                    )}
                </div>
            )}
        </nav>
    )
}