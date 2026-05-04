export const ICE_SERVERS = [
  {
    urls: import.meta.env.VITE_TURN_URL || 'turn:nook.app:3478',
    username: import.meta.env.VITE_TURN_USERNAME || 'nook',
    credential: import.meta.env.VITE_TURN_SECRET || '',
    credentialType: 'password'
  },
  {
    urls: import.meta.env.VITE_STUN_URL || 'stun:nook.app:3478'
  }
];
