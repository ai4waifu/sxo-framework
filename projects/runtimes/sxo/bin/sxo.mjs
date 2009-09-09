#!/usr/bin/env node
import { main } from '../dist/index.js';

const code = await main(process.argv);
process.exitCode = code;
