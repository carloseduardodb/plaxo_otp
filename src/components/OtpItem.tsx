import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Copy, Trash2, Check, Clock } from 'lucide-react';
import { getPlatformIcon, getPlatformColor } from '../utils/platformIcons';

interface Props {
  app: {
    id: string;
    name: string;
    secret: string;
  };
  onDelete: (id: string) => void;
}

export default function OtpItem({ app, onDelete }: Props) {
  const [otp, setOtp] = useState('------');
  const [timeLeft, setTimeLeft] = useState(30);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    const generateOtp = async () => {
      try {
        const code = await invoke<string>('generate_otp', { appId: app.id });
        setOtp(code);
      } catch (error) {
        console.error('Failed to generate OTP:', error);
      }
    };

    const updateTimer = () => {
      const now = Math.floor(Date.now() / 1000);
      const remaining = 30 - (now % 30);
      setTimeLeft(remaining);
      
      if (remaining === 30) {
        generateOtp();
      }
    };

    generateOtp();
    updateTimer();
    
    const interval = setInterval(updateTimer, 1000);
    return () => clearInterval(interval);
  }, [app.id]);

  const copyToClipboard = async () => {
    try {
      await invoke('copy_to_clipboard', { text: otp });
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  };

  const progressPercentage = (timeLeft / 30) * 100;
  const isExpiring = timeLeft <= 10;
  
  const PlatformIcon = getPlatformIcon(app.name);
  const platformColor = getPlatformColor(app.name);

  return (
    <div className="bg-plaxo-background/30 border border-plaxo-border rounded-xl p-4 space-y-4 hover:bg-plaxo-background/50 transition-colors">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div 
            className="flex items-center justify-center w-10 h-10 rounded-xl"
            style={{ backgroundColor: `${platformColor}15`, color: platformColor }}
          >
            <PlatformIcon className="w-5 h-5" />
          </div>
          <h3 className="font-semibold text-plaxo-text truncate">{app.name}</h3>
        </div>
        <button
          onClick={() => onDelete(app.id)}
          className="text-plaxo-text-secondary hover:text-plaxo-error p-1.5 rounded-lg hover:bg-plaxo-error/10 transition-colors"
          title="Remover aplicativo"
        >
          <Trash2 className="w-4 h-4" />
        </button>
      </div>
      
      <div className="space-y-3">
        <div className="text-center">
          <div className="font-mono text-3xl font-bold text-plaxo-primary mb-2 tracking-wider select-all">
            {otp}
          </div>
        </div>
        
        <div className="space-y-2">
          <div className="otp-progress">
            <div 
              className={`otp-progress-bar ${isExpiring ? 'bg-plaxo-warning' : 'bg-plaxo-primary'}`}
              style={{ width: `${progressPercentage}%` }}
            />
          </div>
          
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-1.5 text-sm text-plaxo-text-secondary">
              <Clock className="w-3.5 h-3.5" />
              <span className={isExpiring ? 'text-plaxo-warning' : ''}>{timeLeft}s</span>
            </div>
            <button
              onClick={copyToClipboard}
              className="flex items-center gap-1.5 bg-plaxo-primary hover:bg-plaxo-primary-hover text-plaxo-background px-3 py-1.5 rounded-lg font-medium transition-colors text-sm"
            >
              {copied ? (
                <>
                  <Check className="w-3.5 h-3.5" />
                  Copiado
                </>
              ) : (
                <>
                  <Copy className="w-3.5 h-3.5" />
                  Copiar
                </>
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
