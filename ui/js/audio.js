class AudioPlayer {
    constructor() {
        this.audioCtx = null;
        this.bgmEnabled = true;
        this.sfxEnabled = true;
        this.bgmPlayer = null;
        this.initialized = false;
    }

    init() {
        if (this.initialized) return;
        
        try {
            const AudioContext = window.AudioContext || window.webkitAudioContext;
            this.audioCtx = new AudioContext();
            this.bgmPlayer = new BGMPlayer(this.audioCtx);
            this.initialized = true;
        } catch (e) {
            console.error('Audio initialization failed:', e);
        }
    }

    async resume() {
        if (this.audioCtx && this.audioCtx.state === 'suspended') {
            await this.audioCtx.resume();
        }
    }

    startBGM() {
        if (this.bgmPlayer && this.bgmEnabled) {
            this.bgmPlayer.start();
            console.log('Audio: BGM started');
        }
    }

    stopBGM() {
        if (this.bgmPlayer) {
            this.bgmPlayer.stop();
            console.log('Audio: BGM stopped');
        }
    }

    toggleMusic() {
        this.bgmEnabled = !this.bgmEnabled;
        if (this.bgmPlayer) {
            this.bgmPlayer.toggle(this.bgmEnabled);
        }
    }

    playBounce() {
        if (!this.sfxEnabled || !this.audioCtx) return;
        
        this.resume();
        
        const osc = this.audioCtx.createOscillator();
        const gain = this.audioCtx.createGain();

        osc.type = 'sine';
        osc.frequency.setValueAtTime(150, this.audioCtx.currentTime);
        osc.frequency.exponentialRampToValueAtTime(75, this.audioCtx.currentTime + 0.08);

        gain.gain.setValueAtTime(0.8, this.audioCtx.currentTime);
        gain.gain.exponentialRampToValueAtTime(0.01, this.audioCtx.currentTime + 0.08);

        osc.connect(gain);
        gain.connect(this.audioCtx.destination);

        osc.start();
        osc.stop(this.audioCtx.currentTime + 0.08);
    }

    setVolume(volume) {
        if (this.bgmPlayer) {
            this.bgmPlayer.setVolume(volume);
        }
    }
}

class BGMPlayer {
    constructor(audioCtx) {
        this.audioCtx = audioCtx;
        this.isPlaying = false;
        this.mainGain = audioCtx.createGain();
        this.mainGain.connect(audioCtx.destination);
        this.mainGain.gain.value = 0.15;
        
        this.chords = [
            [130.81, 261.63, 329.63, 392.00, 493.88],
            [220.00, 261.63, 329.63, 392.00],
            [174.61, 261.63, 329.63, 349.23],
            [196.00, 493.88, 293.66, 349.23],
        ];
        this.melody = [261.63, 329.63, 392.00, 493.88, 440.00, 392.00, 329.63, 293.66];
    }

    start() {
        if (this.isPlaying) return;
        this.isPlaying = true;
        this.updateVolume();
        this.playLoop();
    }

    stop() {
        this.isPlaying = false;
        this.updateVolume();
    }

    toggle(enabled) {
        if (enabled) {
            this.start();
        } else {
            this.stop();
        }
    }

    updateVolume() {
        const targetVol = (this.isPlaying) ? 0.15 : 0;
        const now = this.audioCtx.currentTime;
        this.mainGain.gain.cancelScheduledValues(now);
        this.mainGain.gain.linearRampToValueAtTime(targetVol, now + 0.1);
    }

    setVolume(volume) {
        this.mainGain.gain.value = volume;
    }

    playLoop() {
        let startTime = this.audioCtx.currentTime;
        let melDuration = 0.5;
        let beat = 0;

        const nextTick = () => {
            if (!this.isPlaying) return;

            let now = this.audioCtx.currentTime;

            if (this.isPlaying) {
                if (beat % 4 === 0) {
                    let chordIdx = Math.floor(beat / 4) % this.chords.length;
                    this.playChord(this.chords[chordIdx], now, 2.0);
                }

                let melIdx = beat % this.melody.length;
                this.playMelodyNote(this.melody[melIdx], now, melDuration);
            }

            beat++;
            let nextTime = startTime + beat * melDuration;
            let delay = (nextTime - this.audioCtx.currentTime) * 1000;
            setTimeout(nextTick, Math.max(0, delay));
        };

        nextTick();
    }

    playChord(freqs, time, duration) {
        freqs.forEach(f => {
            this.createOscillator(f, 0.05, time, duration, 'sine');
            this.createOscillator(f * 3, 0.02, time, duration, 'sine');
            this.createOscillator(f * 6, 0.01, time, duration, 'sine');
        });
    }

    playMelodyNote(freq, time, duration) {
        this.createOscillator(freq, 0.08, time, duration, 'sine');
        this.createOscillator(freq * 2, 0.03, time, duration, 'sine');
    }

    createOscillator(freq, vol, time, duration, type) {
        const osc = this.audioCtx.createOscillator();
        const g = this.audioCtx.createGain();
        osc.type = type;
        osc.frequency.value = freq;

        g.gain.setValueAtTime(0, time);
        g.gain.linearRampToValueAtTime(vol, time + 0.02);
        g.gain.exponentialRampToValueAtTime(0.001, time + duration);

        osc.connect(g);
        g.connect(this.mainGain);
        osc.start(time);
        osc.stop(time + duration);
    }
}
