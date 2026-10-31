import type React from 'react';
import { Clock, Play, ShieldAlert, Pause } from 'lucide-react';

interface PauseFilterBannerProps {
  isPaused: boolean;
  remainingMin: number;
  remainingSecMod: number;
  isPausing: boolean;
  onPause: (seconds: number) => Promise<void>;
  onUnpause: () => Promise<void>;
  isVi: boolean;
}

export const PauseFilterBanner: React.FC<PauseFilterBannerProps> = ({
  isPaused,
  remainingMin,
  remainingSecMod,
  isPausing,
  onPause,
  onUnpause,
  isVi,
}) => {
  if (isPaused) {
    return (
      <div className="bg-amber-500/10 border border-amber-300 text-amber-950 p-4 rounded-2xl flex flex-wrap items-center justify-between gap-3 shadow-xs">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-amber-500 text-white flex items-center justify-center shrink-0">
            <Clock className="w-5 h-5 animate-pulse" />
          </div>
          <div>
            <h4 className="font-bold text-sm text-amber-900">
              ⏸️ {isVi ? 'Chế độ chặn Web & Quảng cáo đang TẠM DỪNG' : 'Web & DNS Filtering is PAUSED'}
            </h4>
            <p className="text-xs text-amber-800 mt-0.5">
              {isVi ? 'Tự động kích hoạt lại sau ' : 'Automatically resumes in '}
              <span className="font-bold font-mono text-amber-950">
                {remainingMin > 0 ? `${remainingMin}m ` : ''}
                {remainingSecMod}s
              </span>{' '}
              {isVi ? '(Cơ chế Auto-Resume chống quên của Numa).' : '(Numa Anti-Forget Auto-Resume Engine).'}
            </p>
          </div>
        </div>
        <button
          type="button"
          disabled={isPausing}
          onClick={onUnpause}
          className="flex items-center gap-2 px-4 py-2 bg-emerald-600 hover:bg-emerald-700 text-white font-semibold rounded-xl text-xs shadow-xs transition cursor-pointer"
        >
          <Play className="w-4 h-4 fill-white" />
          <span>{isVi ? 'Tiếp Tục Chặn Ngay' : 'Resume Blocking Now'}</span>
        </button>
      </div>
    );
  }

  return (
    <div className="bg-slate-50 border border-slate-200/80 p-3.5 rounded-2xl flex flex-wrap items-center justify-between gap-3 text-xs shadow-xs">
      <div className="flex items-center gap-2.5 text-slate-700">
        <ShieldAlert className="w-4 h-4 text-emerald-600" />
        <span className="font-medium">
          {isVi ? 'Chế độ bảo vệ Web & Lọc DNS đang' : 'Web & DNS Protection is'}{' '}
          <strong className="text-emerald-700 font-bold">
            {isVi ? 'HOẠT ĐỘNG' : 'ACTIVE'}
          </strong>
          .
        </span>
      </div>
      <div className="flex items-center gap-2">
        <span className="text-slate-500 text-[11px] hidden sm:inline">
          {isVi ? 'Cần truy cập tạm thời?' : 'Need temporary access?'}
        </span>
        <button
          type="button"
          disabled={isPausing}
          onClick={() => onPause(900)}
          className="flex items-center gap-1.5 px-3 py-1.5 bg-white hover:bg-slate-100 border border-slate-300 text-slate-700 rounded-lg font-medium text-xs shadow-2xs transition cursor-pointer"
          title={isVi ? 'Tự động bật lại sau 15 phút' : 'Auto-resumes in 15 mins'}
        >
          <Pause className="w-3.5 h-3.5" />
          <span>{isVi ? 'Tạm dừng 15p' : 'Pause 15m'}</span>
        </button>
        <button
          type="button"
          disabled={isPausing}
          onClick={() => onPause(3600)}
          className="flex items-center gap-1.5 px-3 py-1.5 bg-white hover:bg-slate-100 border border-slate-300 text-slate-700 rounded-lg font-medium text-xs shadow-2xs transition cursor-pointer"
          title={isVi ? 'Tự động bật lại sau 1 giờ' : 'Auto-resumes in 1 hour'}
        >
          <Clock className="w-3.5 h-3.5" />
          <span>{isVi ? '1 giờ' : '1 hr'}</span>
        </button>
      </div>
    </div>
  );
};
