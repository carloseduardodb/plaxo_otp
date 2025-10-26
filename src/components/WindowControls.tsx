import { appWindow } from '@tauri-apps/api/window';

export default function WindowControls() {
  const minimize = () => appWindow.minimize();
  const close = () => appWindow.close();

  return (
    <div className="window-controls">
      <button className="window-control" onClick={minimize} title="Minimizar">
        −
      </button>
      <button className="window-control close" onClick={close} title="Fechar">
        ×
      </button>
    </div>
  );
}
