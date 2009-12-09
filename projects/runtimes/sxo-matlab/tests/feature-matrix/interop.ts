import { feature } from '@sxo/harness';

export const interopFeatures = [
    feature('py_list', 'interop')
        .unsupported('SILENT WRONG: py.list([1,2]) → list([1, 2])')
        .pure()
        .gap('py.list', 'py.list([1, 2])', { expected: '...', notes: 'currently list([1, 2])' })
        .done(),
    feature('py_math_sqrt', 'interop')
        .unsupported('SILENT WRONG: py.math.sqrt(4) and math.sqrt(4) both collapse to matlab sqrt → 2')
        .pure()
        .gap('py.math.sqrt', 'py.math.sqrt(4)', { expected: '...', notes: 'currently returns 2 via path strip to sqrt' })
        .done(),
    feature('gpuArray_zeros', 'interop')
        .unsupported('SILENT WRONG: gpuArray.zeros(2) → zeros(2) (package path stripped)')
        .pure()
        .gap('gpuarray.zeros', 'gpuArray.zeros(2)', { expected: '...', notes: 'currently zeros(2)' })
        .done(),
    feature('coder_typeof', 'interop')
        .unsupported('SILENT WRONG: coder.typeof(1) → typeof(1)')
        .pure()
        .gap('coder.typeof', 'coder.typeof(1)', { expected: '...', notes: 'currently typeof(1)' })
        .done(),
    feature('matlab_lang_on', 'interop')
        .unsupported('SILENT WRONG: matlab.lang.OnOffSwitchState.on → on')
        .pure()
        .gap('matlab.lang.on', 'matlab.lang.OnOffSwitchState.on', { expected: '...', notes: 'currently returns on' })
        .done(),
];
