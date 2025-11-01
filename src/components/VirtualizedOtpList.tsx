import { useMemo } from 'react';
import OtpItem from './OtpItem';
import { PERFORMANCE_CONFIG } from '../config/performance';

interface OtpApp {
  id: string;
  name: string;
  secret: string;
}

interface Props {
  apps: OtpApp[];
  onDelete: (id: string) => void;
  onEdit: () => void;
}

export default function VirtualizedOtpList({ apps, onDelete, onEdit }: Props) {
  const visibleApps = useMemo(() => {
    return apps.slice(0, PERFORMANCE_CONFIG.MAX_VISIBLE_APPS);
  }, [apps]);

  const hasMoreApps = apps.length > PERFORMANCE_CONFIG.MAX_VISIBLE_APPS;

  return (
    <div className="max-w-md mx-auto space-y-4">
      {visibleApps.map(app => (
        <OtpItem
          key={app.id}
          app={app}
          onDelete={onDelete}
          onEdit={onEdit}
        />
      ))}

      {hasMoreApps && (
        <div className="text-center text-plaxo-text-secondary text-sm py-4">
          Mostrando {PERFORMANCE_CONFIG.MAX_VISIBLE_APPS} de {apps.length} aplicativos.
          <br />
          Use a busca para encontrar aplicativos específicos.
        </div>
      )}
    </div>
  );
}
