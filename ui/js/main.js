import { Game } from './game.js';

console.log('=== EyeMotion Main Entry ===');

function initGame() {
    console.log('Initializing game...');
    
    if (window.game) {
        console.log('Game already initialized');
        return;
    }
    
    window.game = new Game();
}

// 确保 DOM 加载完成后初始化
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initGame);
} else {
    initGame();
}
