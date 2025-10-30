import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Cloud, CloudOff } from 'lucide-react';

export const GoogleDriveSync: React.FC = () => {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    checkAuth();
  }, []);

  const checkAuth = async () => {
    try {
      const authenticated = await invoke<boolean>('check_google_auth');
      setIsAuthenticated(authenticated);
      if (authenticated) {
        console.log('✅ Google Drive conectado - Sincronização automática ativa!');
      }
    } catch (error) {
      console.log('Google Drive não conectado');
    }
  };

  const handleAuth = async () => {
    try {
      setIsLoading(true);
      await invoke('google_drive_auth_flow');
      setIsAuthenticated(true);
      console.log('✅ Google Drive conectado - Sincronização automática ativa!');
    } catch (error) {
      console.error('Erro na autenticação:', error);
    } finally {
      setIsLoading(false);
    }
  };



  return (
    <div className="flex items-center">
      {!isAuthenticated ? (
        <button
          onClick={handleAuth}
          disabled={isLoading}
          className="flex items-center justify-center w-10 h-10 bg-plaxo-surface hover:bg-plaxo-surface-hover text-plaxo-text-secondary hover:text-plaxo-text border border-plaxo-border rounded-xl transition-colors disabled:opacity-50"
          title="Conectar Google Drive (Sincronização Automática)"
        >
          <CloudOff size={18} />
        </button>
      ) : (
        <div 
          className="flex items-center justify-center w-10 h-10 bg-plaxo-surface border border-plaxo-border rounded-xl"
          title="Google Drive conectado - Sincronização automática ativa"
        >
          <Cloud size={18} className="text-plaxo-primary" />
        </div>
      )}
    </div>
  );
};
