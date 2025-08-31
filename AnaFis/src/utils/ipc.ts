import { EventEmitter } from 'eventemitter3';

export const bus = new EventEmitter(); // process-wide event bus
