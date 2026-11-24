import { describe, expect, it, vi } from "vitest";

import { createWheelPager } from "./wheel-paging";

function wheel(deltaY: number, overrides: Partial<WheelEvent> = {}) {
  return {
    deltaY,
    deltaMode: 0,
    ctrlKey: false,
    metaKey: false,
    ...overrides,
  };
}

describe("createWheelPager", () => {
  it("accumulates small deltas and flips once the threshold is reached", () => {
    let timestamp = 1000;
    const onNext = vi.fn();
    const onPrevious = vi.fn();
    const pager = createWheelPager({
      onNext,
      onPrevious,
      threshold: 50,
      cooldownMs: 300,
      now: () => timestamp,
    });

    expect(pager(wheel(20))).toBe(false);
    timestamp = 1010;
    expect(pager(wheel(20))).toBe(false);
    timestamp = 1020;
    expect(pager(wheel(20))).toBe(true);
    expect(onNext).toHaveBeenCalledTimes(1);
    expect(onPrevious).not.toHaveBeenCalled();
  });

  it("flips backwards for negative deltas", () => {
    const timestamp = 1000;
    const onNext = vi.fn();
    const onPrevious = vi.fn();
    const pager = createWheelPager({
      onNext,
      onPrevious,
      now: () => timestamp,
    });

    expect(pager(wheel(-80))).toBe(true);
    expect(onPrevious).toHaveBeenCalledTimes(1);
    expect(onNext).not.toHaveBeenCalled();
  });

  it("applies a cooldown to avoid multiple pages per gesture", () => {
    let timestamp = 1000;
    const onNext = vi.fn();
    const pager = createWheelPager({
      onNext,
      onPrevious: vi.fn(),
      cooldownMs: 350,
      now: () => timestamp,
    });

    expect(pager(wheel(100))).toBe(true);
    timestamp = 1100;
    expect(pager(wheel(100))).toBe(false);
    timestamp = 1400;
    expect(pager(wheel(100))).toBe(true);
    expect(onNext).toHaveBeenCalledTimes(2);
  });

  it("ignores pinch-zoom wheel events", () => {
    const timestamp = 1000;
    const onNext = vi.fn();
    const pager = createWheelPager({
      onNext,
      onPrevious: vi.fn(),
      now: () => timestamp,
    });

    expect(pager(wheel(200, { ctrlKey: true }))).toBe(false);
    expect(pager(wheel(200, { metaKey: true }))).toBe(false);
    expect(onNext).not.toHaveBeenCalled();
  });

  it("normalizes line-based wheel deltas", () => {
    const timestamp = 1000;
    const onNext = vi.fn();
    const pager = createWheelPager({
      onNext,
      onPrevious: vi.fn(),
      now: () => timestamp,
    });

    expect(pager(wheel(4, { deltaMode: 1 }))).toBe(true);
    expect(onNext).toHaveBeenCalledTimes(1);
  });
});
