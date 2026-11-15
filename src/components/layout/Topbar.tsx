export function Topbar({ title }: { title: string }) {
  return (
    <header className="topbar">
      <div className="breadcrumb">
        Kito Studio <span>/</span> {title}
      </div>
      <div className="topbar-right">
        <span className="local-pill">
          <i /> Projet local
        </span>
        <div className="avatar">K</div>
      </div>
    </header>
  );
}
