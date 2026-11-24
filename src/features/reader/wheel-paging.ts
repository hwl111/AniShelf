export interface WheelLikeEvent {
  deltaY: number;
  deltaMode: number;
  ctrlKey: boolean;
  metaKey: boolean;
}

interface WheelPagerOptions {
  onNext: () => void;
  onPrevious: () => void;
  threshold?: number;
  cooldownMs?: number;
  now?: () => number;
}

const RESET_AFTER_MS = 250;

export function createWheelPager({
  onNext,
  onPrevious,
  threshold = 50,
  cooldownMs = 350,
  now = () => performance.now(),
}: WheelPagerOptions): (event: WheelLikeEvent) => boolean {
  let accumulated = 0;
  let lastEventAt = 0;
  let lastFlipAt = 0;

  return (event) => {
    if (event.ctrlKey || event.metaKey) {
      return false;
    }

    const timestamp = now();
    if (timestamp - lastEventAt > RESET_AFTER_MS) {
      accumulated = 0;
    }
    lastEventAt = timestamp;

    const multiplier = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? 400 : 1;
    accumulated += event.deltaY * multiplier;

    if (timestamp - lastFlipAt < cooldownMs) {
      return false;
    }
    if (Math.abs(accumulated) < threshold) {
      return false;
    }

    const direction = accumulated > 0 ? 1 : -1;
    accumulated = 0;
    lastFlipAt = timestamp;

    if (direction > 0) {
      onNext();
    } else {
      onPrevious();
    }
    return true;
  };
}
