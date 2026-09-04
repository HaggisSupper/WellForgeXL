import { EmptyCanvas } from "./components/EmptyCanvas";
import { Sidebar } from "./components/Sidebar";

export function App() {
  return (
    <div className="flex min-h-screen bg-slate-900 font-sans text-slate-100">
      <Sidebar />
      <EmptyCanvas />
    </div>
  );
}
