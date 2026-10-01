// Audio utilities - Silent mode (Audio disabled per operator policy)

export function setSoundEnabled(_enabled: boolean): void {}

export function isSoundEnabled(): boolean {
  return false;
}

export function playUiClick(): void {}

export function playWarningChime(): void {}

export function playCriticalAlarm(): void {}
