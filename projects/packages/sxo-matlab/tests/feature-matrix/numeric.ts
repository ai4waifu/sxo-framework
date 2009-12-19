import { feature } from '@sxo/harness';

export const numericFeatures = [
    feature('complex', 'numeric')
        .unsupported('oak bad literal on 2i')
        .pure()
        .gap('complex.i', '1+2i', { expected: '1+2i' })
        .gap('complex.real', 'real(1+2i)', { expected: '1' })
        .done(),
    feature('imag_unit_literal', 'numeric')
        .unsupported('oak bad literal on bare 1i (also 2i in complex entry)')
        .pure()
        .gap('imag.1i', '1i', { expected: '1i' })
        .done(),
    feature('complex_ctor', 'numeric').unsupported().pure().gap('complex.ctor', 'complex(1, 2)', { expected: '1+2i' }).done(),
    feature('single', 'numeric').unsupported().pure().gap('single.1', 'single(1)', { expected: '1' }).done(),
    feature('floor', 'numeric').unsupported().pure().gap('floor.2_7', 'floor(2.7)', { expected: '2' }).done(),
    feature('mod', 'numeric').unsupported().pure().gap('mod.10_3', 'mod(10, 3)', { expected: '1' }).done(),
    feature('hypot', 'numeric').unsupported().pure().gap('hypot.34', 'hypot(3, 4)', { expected: '5' }).done(),
    feature('ieee_edge', 'numeric')
        .partial('SILENT WRONG: 0/0→0 and Inf-Inf→0 (MATLAB expects NaN); 0^0→1 matches MATLAB')
        .pure()
        .gap('ieee.0over0', '0/0', { expected: 'NaN', notes: 'currently 0' })
        .gap('ieee.inf_minus_inf', 'Inf - Inf', { expected: 'NaN', notes: 'currently 0' })
        .eval('ieee.0pow0', '0^0', '1', { notes: 'MATLAB-compatible' })
        .done(),
    feature('i_squared', 'numeric')
        .unsupported('bare i/j symbols retained; 1+2i / 1i still oak bad literal')
        .pure()
        .gap('i.sq', 'i^2', { expected: '-1' })
        .gap('j.sq', 'j^2', { expected: '-1' })
        .done(),
    feature('hex2dec', 'numeric').unsupported().pure().gap('hex2dec.ff', "hex2dec('FF')", { expected: '255' }).done(),
    feature('dec2hex', 'numeric').unsupported().pure().gap('dec2hex.255', 'dec2hex(255)', { expected: "'FF'" }).done(),
    feature('log2', 'numeric').unsupported().pure().gap('log2.8', 'log2(8)', { expected: '3' }).done(),
    feature('pow2', 'numeric').unsupported().pure().gap('pow2.3', 'pow2(3)', { expected: '8' }).done(),
    feature('nextpow2', 'numeric').unsupported().pure().gap('nextpow2.5', 'nextpow2(5)', { expected: '3' }).done(),
];
