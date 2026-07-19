import { BrowserRouter, Routes, Route, NavLink } from 'react-router-dom';
import Home from './pages/Home';
import Libraries from './pages/Libraries';
import LibraryDetail from './pages/LibraryDetail';
import Jobs from './pages/Jobs';
import Settings from './pages/Settings';
import Search from './pages/Search';
import Selects from './pages/Selects';
import Entities from './pages/Entities';

function Layout() {
  const navClass =
    'flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium transition-colors';
  const activeClass = 'bg-brand-100 text-brand-700 dark:bg-brand-900 dark:text-brand-100';
  const inactiveClass = 'text-neutral-600 hover:bg-neutral-100 dark:text-neutral-300 dark:hover:bg-neutral-800';

  return (
    <div className="flex h-full w-full">
      <aside className="flex w-56 flex-col border-r border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-950">
        <div className="flex h-14 items-center border-b border-neutral-200 px-4 dark:border-neutral-800">
          <span className="text-lg font-bold tracking-tight">SceneWeaver</span>
        </div>
        <nav className="flex flex-1 flex-col gap-1 p-3">
          <NavLink to="/" className={({ isActive }) => `${navClass} ${isActive ? activeClass : inactiveClass}`}>
            <span>🏠</span> 首页
          </NavLink>
          <NavLink to="/libraries" className={({ isActive }) => `${navClass} ${isActive ? activeClass : inactiveClass}`}>
            <span>🎞️</span> 素材库
          </NavLink>
          <NavLink to="/search" className={({ isActive }) => `${navClass} ${isActive ? activeClass : inactiveClass}`}>
            <span>🔎</span> 搜索
          </NavLink>
          <NavLink to="/jobs" className={({ isActive }) => `${navClass} ${isActive ? activeClass : inactiveClass}`}>
            <span>⚙️</span> 任务
          </NavLink>
          <NavLink to="/selects" className={({ isActive }) => `${navClass} ${isActive ? activeClass : inactiveClass}`}>
            <span>⭐</span> 选片
          </NavLink>
          <NavLink to="/entities" className={({ isActive }) => `${navClass} ${isActive ? activeClass : inactiveClass}`}><span>👤</span> 实体</NavLink>
          <NavLink to="/settings" className={({ isActive }) => `${navClass} ${isActive ? activeClass : inactiveClass}`}>
            <span>🔧</span> 设置
          </NavLink>
        </nav>
        <div className="border-t border-neutral-200 p-3 text-xs text-neutral-500 dark:border-neutral-800">
          v0.1.0 · 本地优先
        </div>
      </aside>
      <main className="flex-1 overflow-auto bg-neutral-50 dark:bg-neutral-900">
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/libraries" element={<Libraries />} />
          <Route path="/libraries/:id" element={<LibraryDetail />} />
          <Route path="/search" element={<Search />} />
          <Route path="/jobs" element={<Jobs />} />
          <Route path="/selects" element={<Selects />} />
          <Route path="/entities" element={<Entities />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </main>
    </div>
  );
}

function App() {
  return (
    <BrowserRouter>
      <Layout />
    </BrowserRouter>
  );
}

export default App;
