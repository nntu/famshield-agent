import {
  Cloud,
  Database,
  Globe,
  Lock,
  Server,
  ShieldAlert,
  ShieldCheck,
  Sliders,
  Smartphone,
} from 'lucide-react';
import type React from 'react';

export type SecuritySubTab = 'auth' | 'cloud' | 'network' | 'maintenance';

interface SecurityTabNavProps {
  activeSubTab: SecuritySubTab;
  setActiveSubTab: (tab: SecuritySubTab) => void;
  totpEnabled: boolean;
  operatingMode?: string;
  isVi: boolean;
}

export const SecurityTabNav: React.FC<SecurityTabNavProps> = ({
  activeSubTab,
  setActiveSubTab,
  totpEnabled,
  operatingMode = 'StandaloneFree',
  isVi,
}) => {
  const isOnline = operatingMode === 'OnlineManaged';

  return (
    <div className="bg-white rounded-2xl border border-slate-200/80 p-2 shadow-xs">
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-2">
        {/* SubTab 1: Auth & Protection */}
        <button
          type="button"
          onClick={() => setActiveSubTab('auth')}
          className={`flex items-center gap-3 p-3 rounded-xl text-left transition-all cursor-pointer ${
            activeSubTab === 'auth'
              ? 'bg-sky-50 text-sky-900 border border-sky-300 ring-2 ring-sky-100 shadow-xs'
              : 'hover:bg-slate-50 text-slate-600 border border-transparent'
          }`}
        >
          <div
            className={`w-10 h-10 rounded-xl flex items-center justify-center shrink-0 transition ${
              activeSubTab === 'auth'
                ? 'bg-sky-600 text-white shadow-2xs'
                : 'bg-slate-100 text-slate-500'
            }`}
          >
            <ShieldCheck className="w-5 h-5" />
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-center justify-between gap-1">
              <span className="font-bold text-xs truncate">
                {isVi ? '1. Tài Khoản & Bảo Mật' : '1. Account & Security'}
              </span>
              <span
                className={`text-[10px] font-bold px-2 py-0.5 rounded-full ${
                  totpEnabled
                    ? 'bg-emerald-100 text-emerald-800'
                    : 'bg-amber-100 text-amber-800'
                }`}
              >
                {totpEnabled ? '2FA: BẬT' : (isVi ? '2FA: CHƯA BẬT' : '2FA: OFF')}
              </span>
            </div>
            <p className="text-[11px] text-slate-500 truncate mt-0.5">
              {isVi ? '2FA TOTP, Master PIN, Chế độ bảo vệ' : '2FA TOTP, Master PIN, Protection mode'}
            </p>
          </div>
        </button>

        {/* SubTab 2: Cloud & Pairing */}
        <button
          type="button"
          onClick={() => setActiveSubTab('cloud')}
          className={`flex items-center gap-3 p-3 rounded-xl text-left transition-all cursor-pointer ${
            activeSubTab === 'cloud'
              ? 'bg-sky-50 text-sky-900 border border-sky-300 ring-2 ring-sky-100 shadow-xs'
              : 'hover:bg-slate-50 text-slate-600 border border-transparent'
          }`}
        >
          <div
            className={`w-10 h-10 rounded-xl flex items-center justify-center shrink-0 transition ${
              activeSubTab === 'cloud'
                ? 'bg-sky-600 text-white shadow-2xs'
                : 'bg-slate-100 text-slate-500'
            }`}
          >
            <Cloud className="w-5 h-5" />
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-center justify-between gap-1">
              <span className="font-bold text-xs truncate">
                {isVi ? '2. Đám Mây & Ghép Nối' : '2. Cloud & Pairing'}
              </span>
              <span
                className={`text-[10px] font-bold px-2 py-0.5 rounded-full ${
                  isOnline
                    ? 'bg-sky-100 text-sky-800'
                    : 'bg-slate-100 text-slate-600'
                }`}
              >
                {isOnline ? 'Online' : 'Offline'}
              </span>
            </div>
            <p className="text-[11px] text-slate-500 truncate mt-0.5">
              {isVi ? 'Máy chủ trung tâm, Mã ghép nối, Cập nhật' : 'Central server, Enrollment, Updates'}
            </p>
          </div>
        </button>

        {/* SubTab 3: Network & System */}
        <button
          type="button"
          onClick={() => setActiveSubTab('network')}
          className={`flex items-center gap-3 p-3 rounded-xl text-left transition-all cursor-pointer ${
            activeSubTab === 'network'
              ? 'bg-sky-50 text-sky-900 border border-sky-300 ring-2 ring-sky-100 shadow-xs'
              : 'hover:bg-slate-50 text-slate-600 border border-transparent'
          }`}
        >
          <div
            className={`w-10 h-10 rounded-xl flex items-center justify-center shrink-0 transition ${
              activeSubTab === 'network'
                ? 'bg-sky-600 text-white shadow-2xs'
                : 'bg-slate-100 text-slate-500'
            }`}
          >
            <Globe className="w-5 h-5" />
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-center justify-between gap-1">
              <span className="font-bold text-xs truncate">
                {isVi ? '3. Mạng & Hệ Thống' : '3. Network & System'}
              </span>
              <span className="text-[10px] font-bold px-2 py-0.5 rounded-full bg-slate-100 text-slate-600">
                127.0.0.2:53
              </span>
            </div>
            <p className="text-[11px] text-slate-500 truncate mt-0.5">
              {isVi ? 'DNS Upstream, Cache, Từ khóa nhạy cảm' : 'DNS Upstream, Cache, Keywords engine'}
            </p>
          </div>
        </button>

        {/* SubTab 4: Data & Maintenance */}
        <button
          type="button"
          onClick={() => setActiveSubTab('maintenance')}
          className={`flex items-center gap-3 p-3 rounded-xl text-left transition-all cursor-pointer ${
            activeSubTab === 'maintenance'
              ? 'bg-rose-50 text-rose-950 border border-rose-300 ring-2 ring-rose-100 shadow-xs'
              : 'hover:bg-slate-50 text-slate-600 border border-transparent'
          }`}
        >
          <div
            className={`w-10 h-10 rounded-xl flex items-center justify-center shrink-0 transition ${
              activeSubTab === 'maintenance'
                ? 'bg-rose-600 text-white shadow-2xs'
                : 'bg-slate-100 text-slate-500'
            }`}
          >
            <Database className="w-5 h-5" />
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-center justify-between gap-1">
              <span className="font-bold text-xs truncate">
                {isVi ? '4. Dữ Liệu & Khôi Phục' : '4. Data & Maintenance'}
              </span>
              <span className="text-[10px] font-bold px-2 py-0.5 rounded-full bg-slate-100 text-slate-600">
                {isVi ? 'Bảo Trì' : 'Tools'}
              </span>
            </div>
            <p className="text-[11px] text-slate-500 truncate mt-0.5">
              {isVi ? 'Sao lưu, Khôi phục, Tác giả, Gỡ cài đặt' : 'Backup, Restore, Author, Uninstall'}
            </p>
          </div>
        </button>
      </div>
    </div>
  );
};
