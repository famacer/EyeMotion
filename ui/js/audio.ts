export class AudioPlayer {
    private audioCtx: AudioContext | null = null;
    public bgmEnabled: boolean = true;
    public sfxEnabled: boolean = true;
    private bgmPlayer: BGMPlayer | null = null;
    private initialized: boolean = false;

    constructor() {
        this.audioCtx = null;
        this.bgmEnabled = true;
        this.sfxEnabled = true;
        this.bgmPlayer = null;
        this.initialized = false;
    }

    init(): void {
        if (this.initialized) return;
        
        try {
            const AudioContextClass = (window.AudioContext || (window as any).webkitAudioContext) as typeof AudioContext;
            if (!AudioContextClass) throw new Error('AudioContext not supported');
            
            this.audioCtx = new AudioContextClass();
            this.bgmPlayer = new BGMPlayer(this.audioCtx);
            this.initialized = true;
            console.log('Audio: Initialized');
        } catch (e) {
            console.error('Audio initialization failed:', e);
            this.initialized = true;
        }
    }

    async resume(): Promise<void> {
        if (this.audioCtx && this.audioCtx.state === 'suspended') {
            try {
                await this.audioCtx.resume();
            } catch (e) {
                console.error('Audio: Failed to resume:', e);
            }
        }
    }

    startBGM(): void {
        if (!this.initialized || !this.bgmPlayer) return;
        if (this.bgmEnabled) this.bgmPlayer.start();
    }

    stopBGM(): void {
        if (this.bgmPlayer) this.bgmPlayer.stop();
    }

    toggleMusic(): void {
        this.bgmEnabled = !this.bgmEnabled;
        if (this.bgmPlayer) this.bgmPlayer.toggle(this.bgmEnabled);
    }

    playBounce(): void {
        if (!this.sfxEnabled || !this.audioCtx) return;
        this.resume();
        
        try {
            const osc = this.audioCtx.createOscillator();
            const gain = this.audioCtx.createGain();

            osc.type = 'sine';
            osc.frequency.setValueAtTime(150, this.audioCtx.currentTime);
            osc.frequency.exponentialRampToValueAtTime(75, this.audioCtx.currentTime + 0.08);

            gain.gain.setValueAtTime(0.2, this.audioCtx.currentTime);
            gain.gain.exponentialRampToValueAtTime(0.01, this.audioCtx.currentTime + 0.08);

            osc.connect(gain);
            gain.connect(this.audioCtx.destination);

            osc.start();
            osc.stop(this.audioCtx.currentTime + 0.08);
        } catch (e) {
            console.error('Audio: Failed to play bounce sound:', e);
        }
    }

    playClick(): void {
        if (!this.sfxEnabled || !this.audioCtx) return;
        this.resume();
        
        try {
            const osc = this.audioCtx.createOscillator();
            const gain = this.audioCtx.createGain();

            osc.type = 'sine';
            // 一个清脆的高音点击声
            osc.frequency.setValueAtTime(800, this.audioCtx.currentTime);
            osc.frequency.exponentialRampToValueAtTime(400, this.audioCtx.currentTime + 0.05);

            gain.gain.setValueAtTime(0.15, this.audioCtx.currentTime);
            gain.gain.exponentialRampToValueAtTime(0.01, this.audioCtx.currentTime + 0.05);

            osc.connect(gain);
            gain.connect(this.audioCtx.destination);

            osc.start();
            osc.stop(this.audioCtx.currentTime + 0.05);
        } catch (e) {
            console.error('Audio: Failed to play click sound:', e);
        }
    }

    setVolume(volume: number): void {
        if (this.bgmPlayer) this.bgmPlayer.setVolume(volume);
    }
}

class BGMPlayer {
    private audioCtx: AudioContext;
    private isPlaying: boolean = false;
    private mainGain: GainNode;
    private loopTimeout: number | null = null;
    private chords = [
        [130.81, 261.63, 329.63, 392.00], // Cmaj7
        [146.83, 293.66, 349.23, 440.00], // Dm7
        [164.81, 329.63, 392.00, 493.88], // Em7
        [174.61, 349.23, 440.00, 523.25], // Fmaj7
        [196.00, 392.00, 493.88, 587.33], // G7
        [220.00, 440.00, 523.25, 659.25], // Am7
        [174.61, 349.23, 440.00, 523.25], // Fmaj7
        [196.00, 392.00, 493.88, 587.33], // G7
    ];
    private melody = [
        // 第一小节
        523.25, 0, 523.25, 587.33, 659.25, 0, 659.25, 587.33,
        // 第二小节
        523.25, 0, 493.88, 0, 440.00, 0, 392.00, 0,
        // 第三小节
        392.00, 0, 523.25, 0, 659.25, 0, 783.99, 0,
        // 第四小节
        880.00, 783.99, 659.25, 523.25, 587.33, 0, 0, 0
    ]; 

    constructor(audioCtx: AudioContext) {
        this.audioCtx = audioCtx;
        this.mainGain = audioCtx.createGain();
        this.mainGain.connect(audioCtx.destination);
        this.mainGain.gain.value = 0.5;
    }

    start(): void {
        if (this.isPlaying) return;
        this.isPlaying = true;
        this.updateVolume();
        this.playLoop();
    }

    stop(): void {
        this.isPlaying = false;
        if (this.loopTimeout) {
            window.clearTimeout(this.loopTimeout);
            this.loopTimeout = null;
        }
        this.updateVolume();
    }

    toggle(enabled: boolean): void {
        enabled ? this.start() : this.stop();
    }

    updateVolume(): void {
        const targetVol = this.isPlaying ? 0.5 : 0;
        const now = this.audioCtx.currentTime;
        this.mainGain.gain.cancelScheduledValues(now);
        this.mainGain.gain.linearRampToValueAtTime(targetVol, now + 0.1);
    }

    setVolume(volume: number): void {
        this.mainGain.gain.value = volume;
    }

    private playLoop(): void {
        let startTime = this.audioCtx.currentTime;
        let beatDuration = 0.45; 
        let beat = 0;

        const nextTick = () => {
            if (!this.isPlaying) return;
            const now = this.audioCtx.currentTime;
            
            const chordIdx = Math.floor(beat / 8) % this.chords.length;
            const currentChord = this.chords[chordIdx];

            // 1. 节奏部分 (Rhythm Section)
            // 仅保留底鼓和踏板，去掉军鼓，保持背景干净
            if (beat % 2 === 0) {
                this.createKick(now);
            }
            this.createHiHat(now);

            // 2. 低音声部 (Bass Section)
            // 贝斯 (Bass) - 增强低频
            if (beat % 2 === 0) {
                this.createInstrumentNote(currentChord[0] * 0.5, 0.12, now, beatDuration * 1.0, 'triangle', {
                    attack: 0.05, decay: 0.1, sustain: 0.5, release: 0.2, filterFreq: 250
                });
            }
            // 大提琴 (Cello) - 极其连贯的长音
            if (beat % 8 === 0) {
                this.createInstrumentNote(currentChord[0] * 0.5, 0.1, now, beatDuration * 8.5, 'sawtooth', {
                    attack: 1.5, decay: 0.5, sustain: 0.8, release: 2.0, vibrato: true, filterFreq: 400
                });
            }

            // 3. 和声伴奏 (Harmony Section)
            // 电吉他 (Electric Guitar) - 移至中低声部，增加厚度
            const guitarFreq = currentChord[beat % 4]; // 降低一个八度
            this.createInstrumentNote(guitarFreq, 0.04, now, beatDuration * 2.0, 'square', {
                attack: 0.1, decay: 0.5, sustain: 0.4, release: 1.0, filterFreq: 800
            });

            // 4. 主奏声部 (Main Section)
            // 钢琴 (Piano) - 增加 Sustain 使音符连贯
            const melIdx = beat % this.melody.length;
            const pianoFreq = this.melody[melIdx];
            if (pianoFreq > 0) {
                this.createInstrumentNote(pianoFreq, 0.12, now, beatDuration * 1.5, 'triangle', {
                    attack: 0.01, decay: 0.3, sustain: 0.4, release: 0.8, filterFreq: 2500
                });
            }

            // 5. 旋律线 (Melody Lines - Crystal & Soft Lead)
            // 彻底重构音色：移除锯齿波，使用更清澈的合成音色，消除“阴间”感
            // 旋律线 A - 水晶三角波 (Crystal Triangle)
            if (beat % 4 === 0) {
                const v1Freq = currentChord[2] * 2;
                this.createInstrumentNote(v1Freq, 0.08, now, beatDuration * 6.0, 'triangle', {
                    attack: 0.2, decay: 0.3, sustain: 0.7, release: 2.0, vibrato: false, filterFreq: 3000 // 高截止频率，保持清澈
                });
            }
            // 旋律线 B - 柔和方波 (Soft Square)
            if ((beat + 2) % 8 === 0) {
                const v2Freq = currentChord[1] * 2;
                this.createInstrumentNote(v2Freq, 0.05, now, beatDuration * 7.0, 'square', {
                    attack: 0.3, decay: 0.4, sustain: 0.6, release: 2.5, vibrato: false, filterFreq: 1500 // 过滤掉高频，使其圆润
                });
            }

            beat++;
            const nextTime = startTime + beat * beatDuration;
            const delay = (nextTime - this.audioCtx.currentTime) * 1000;
            this.loopTimeout = window.setTimeout(nextTick, Math.max(0, delay));
        };

        nextTick();
    }

    private createKick(time: number): void {
        const osc = this.audioCtx.createOscillator();
        const g = this.audioCtx.createGain();
        osc.type = 'sine';
        osc.frequency.setValueAtTime(120, time);
        osc.frequency.exponentialRampToValueAtTime(0.01, time + 0.2);
        g.gain.setValueAtTime(0.4, time);
        g.gain.exponentialRampToValueAtTime(0.01, time + 0.2);
        osc.connect(g);
        g.connect(this.mainGain);
        osc.start(time);
        osc.stop(time + 0.2);
    }

    private createHiHat(time: number): void {
        const noise = this.audioCtx.createBufferSource();
        const bufferSize = this.audioCtx.sampleRate * 0.05;
        const buffer = this.audioCtx.createBuffer(1, bufferSize, this.audioCtx.sampleRate);
        const data = buffer.getChannelData(0);
        for (let i = 0; i < bufferSize; i++) {
            data[i] = Math.random() * 2 - 1;
        }
        noise.buffer = buffer;

        const noiseFilter = this.audioCtx.createBiquadFilter();
        noiseFilter.type = 'highpass';
        noiseFilter.frequency.setValueAtTime(8000, time);

        const g = this.audioCtx.createGain();
        g.gain.setValueAtTime(0.05, time);
        g.gain.exponentialRampToValueAtTime(0.01, time + 0.05);

        noise.connect(noiseFilter);
        noiseFilter.connect(g);
        g.connect(this.mainGain);
        noise.start(time);
        noise.stop(time + 0.05);
    }

    // 通用乐器音色合成方法
    private createInstrumentNote(
        freq: number, 
        gainVal: number, 
        time: number, 
        duration: number, 
        type: OscillatorType,
        params: { 
            attack: number, 
            decay: number, 
            sustain: number, 
            release: number, 
            vibrato?: boolean,
            filterFreq: number
        }
    ): void {
        const osc = this.audioCtx.createOscillator();
        const g = this.audioCtx.createGain();
        const filter = this.audioCtx.createBiquadFilter();

        osc.type = type;
        osc.frequency.setValueAtTime(freq, time);

        // 颤音 (Vibrato)
        if (params.vibrato) {
            const lfo = this.audioCtx.createOscillator();
            const lfoGain = this.audioCtx.createGain();
            lfo.frequency.value = 5; // 5Hz 颤音
            lfoGain.gain.value = freq * 0.01; // 1% 频率偏移
            lfo.connect(lfoGain);
            lfoGain.connect(osc.frequency);
            lfo.start(time);
            lfo.stop(time + duration);
        }

        filter.type = 'lowpass';
        filter.frequency.setValueAtTime(params.filterFreq, time);
        filter.frequency.exponentialRampToValueAtTime(params.filterFreq * 0.5, time + duration);

        // ADSR 包络
        g.gain.setValueAtTime(0, time);
        g.gain.linearRampToValueAtTime(gainVal, time + params.attack);
        g.gain.linearRampToValueAtTime(gainVal * params.sustain, time + params.attack + params.decay);
        g.gain.exponentialRampToValueAtTime(0.0001, time + duration);

        osc.connect(filter);
        filter.connect(g);
        g.connect(this.mainGain);

        osc.start(time);
        osc.stop(time + duration);
    }
}
