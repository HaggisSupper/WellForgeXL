import { useEffect, useState } from "react";

interface TransitionCountdownProps {
  initialSeconds?: number;
  reason?: string;
}

export function TransitionCountdown({ initialSeconds = 19, reason = "Dwell timer active" }: TransitionCountdownProps) {
  const [secondsLeft, setSecondsLeft] = useState(initialSeconds);

  useEffect(() => {
    setSecondsLeft(initialSeconds);
    const timer = setInterval(() => {
      setSecondsLeft((prev) => (prev > 0 ? prev - 1 : initialSeconds));
    }, 1000);

    return () => clearInterval(timer);
  }, [initialSeconds]);

  const percentage = Math.round((secondsLeft / initialSeconds) * 100);

  return (
    <div className="dwell-timer-card">
      <div className="timer-ring-container">
        <svg viewBox="0 0 36 36" className="timer-svg">
          <path
            className="ring-bg"
            d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
          />
          <path
            className="ring-progress"
            strokeDasharray={`${percentage}, 100`}
            d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
          />
        </svg>
        <span className="timer-number tabular-num">{secondsLeft}s</span>
      </div>
      <div className="timer-info">
        <p className="timer-title">Transition Classification Dwell</p>
        <p className="timer-subtitle">{reason}</p>
      </div>
    </div>
  );
}
