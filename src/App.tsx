import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Shield, Lock } from 'lucide-react';
import MasterPasswordModal from './components/MasterPasswordModal';
import OtpItem from './components/OtpItem';
import SearchBar from './components/SearchBar';
import AddAppModal from './components/AddAppModal';
import ImportModal from './components/ImportModal';
import ConfirmModal from './components/ConfirmModal';
import WindowControls from './components/WindowControls';

interface OtpApp {
  id: string;
  name: string;
  secret: string;
}

function App() {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [apps, setApps] = useState<OtpApp[]>([]);
  const [searchTerm, setSearchTerm] = useState('');
  const [showAddModal, setShowAddModal] = useState(false);
  const [showImportModal, setShowImportModal] = useState(false);
  const [showResetModal, setShowResetModal] = useState(false);
  const [hasMasterPassword, setHasMasterPassword] = useState(false);

  useEffect(() => {
    const checkMasterPassword = async () => {
      try {
        const exists = await invoke<boolean>('has_master_password');
        setHasMasterPassword(exists);
      } catch (error) {
        console.error('Failed to check master password:', error);
      }
    };
    checkMasterPassword();
  }, []);

  const loadApps = async () => {
    try {
      const loadedApps = await invoke<OtpApp[]>('get_apps');
      setApps(loadedApps);
    } catch (error) {
      console.error('Failed to load apps:', error);
    }
  };

  const handleMasterPassword = async (password: string) => {
    try {
      const isValid = await invoke<boolean>('verify_master_password', { password });
      if (isValid) {
        setIsAuthenticated(true);
        await loadApps();
      }
      return isValid;
    } catch (error) {
      console.error('Failed to verify password:', error);
      return false;
    }
  };

  const handleReset = async () => {
    try {
      await invoke('reset_master_password');
      setIsAuthenticated(false);
      setApps([]);
      setHasMasterPassword(false);
      setShowResetModal(false);
    } catch (error) {
      console.error('Failed to reset:', error);
    }
  };

  const handleAddApp = async (name: string, secret: string) => {
    try {
      await invoke('add_app', { name, secret });
      await loadApps();
    } catch (error) {
      console.error('Failed to add app:', error);
      throw error;
    }
  };

  const handleDeleteApp = async (id: string) => {
    try {
      await invoke('delete_app', { appId: id });
      await loadApps();
    } catch (error) {
      console.error('Failed to delete app:', error);
    }
  };

  const filteredApps = apps.filter(app =>
    app.name.toLowerCase().includes(searchTerm.toLowerCase())
  );

  if (!isAuthenticated) {
    return (
      <div className="auth-container">
        <MasterPasswordModal 
          onSubmit={handleMasterPassword} 
          onReset={() => setShowResetModal(true)}
          isFirstTime={!hasMasterPassword} 
        />
        
        <ConfirmModal
          isOpen={showResetModal}
          title="Resetar Senha Mestre"
          message="Esta ação irá apagar TODOS os seus dados e aplicativos cadastrados. Você terá que configurar uma nova senha mestre e adicionar todos os aplicativos novamente. Esta ação não pode ser desfeita."
          confirmText="Sim, resetar tudo"
          cancelText="Cancelar"
          variant="danger"
          onConfirm={handleReset}
          onCancel={() => setShowResetModal(false)}
        />
      </div>
    );
  }

  return (
    <div className="main-container">
      <div className="content-container">
        <div className="header-sticky">
          <div className="flex items-center justify-center gap-3 mb-4">
            <div className="flex items-center justify-center w-10 h-10 bg-plaxo-primary/10 rounded-xl">
              <Shield className="w-5 h-5 text-plaxo-primary" />
            </div>
            <h1 className="text-xl font-heading font-bold text-plaxo-text">
              Plaxo OTP
            </h1>
          </div>
          
          <SearchBar
            value={searchTerm}
            onChange={setSearchTerm}
            onAddClick={() => setShowAddModal(true)}
            onImportClick={() => setShowImportModal(true)}
          />
        </div>
        
        <div className="content-scrollable">
          {filteredApps.length === 0 ? (
            <div className="flex items-center justify-center h-full">
              <div className="text-center text-plaxo-text-secondary">
                <div className="inline-flex items-center justify-center w-16 h-16 bg-plaxo-border/20 rounded-2xl mb-4">
                  <Lock className="w-8 h-8" />
                </div>
                <p className="text-lg font-medium mb-2">Nenhum aplicativo cadastrado</p>
                <p className="text-sm">Adicione seu primeiro aplicativo para começar a gerar códigos OTP</p>
              </div>
            </div>
          ) : (
            <div className="max-w-md mx-auto space-y-4">
              {filteredApps.map(app => (
                <OtpItem
                  key={app.id}
                  app={app}
                  onDelete={handleDeleteApp}
                  onEdit={loadApps}
                />
              ))}
            </div>
          )}
        </div>

        {showAddModal && (
          <AddAppModal
            onSubmit={handleAddApp}
            onClose={() => setShowAddModal(false)}
          />
        )}

        {showImportModal && (
          <ImportModal
            onClose={() => setShowImportModal(false)}
            onImportComplete={loadApps}
          />
        )}
      </div>
    </div>
  );
}

export default App;
