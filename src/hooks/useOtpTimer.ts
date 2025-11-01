import { useState, useEffect, useRef } from "react";

interface UseOtpTimerReturn {
  timeLeft: number;
  shouldRefresh: boolean;
}

export function useOtpTimer(): UseOtpTimerReturn {
  const [timeLeft, setTimeLeft] = useState(30);
  const [shouldRefresh, setShouldRefresh] = useState(false);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    const updateTimer = () => {
      const now = Math.floor(Date.now() / 1000);
      const remaining = 30 - (now % 30);
      setTimeLeft(remaining);

      if (remaining === 30) {
        setShouldRefresh(true);
        setTimeout(() => setShouldRefresh(false), 100);
      }
    };

    updateTimer();
    intervalRef.current = setInterval(updateTimer, 1000);

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, []);

  return { timeLeft, shouldRefresh };
}
