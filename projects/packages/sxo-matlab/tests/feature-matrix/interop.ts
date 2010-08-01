import { feature } from '@sxo/harness';

export const interopFeatures = [
    feature('py_list', 'interop')
        .unsupported('parse keeps Member call path via oak; py.list runtime still open (was silent → list([...]))')
        .pure()
        .gap('py.list', 'py.list([1, 2])', {
            expected: '...',
            notes: 'Form Application(Member[py, list], …); eval Reject',
        })
        .done(),
    feature('py_math_sqrt', 'interop')
        .unsupported('parse keeps nested Member path; py.math.sqrt runtime still open (was silent → 2)')
        .pure()
        .gap('py.math.sqrt', 'py.math.sqrt(4)', {
            expected: '...',
            notes: 'Form keeps py.math.sqrt chain; eval Reject',
        })
        .done(),
    feature('gpuArray_zeros', 'interop')
        .unsupported('parse keeps Member call; gpuArray.zeros runtime still open (was silent → zeros(2))')
        .pure()
        .gap('gpuarray.zeros', 'gpuArray.zeros(2)', {
            expected: '...',
            notes: 'Form Application(Member[gpuArray, zeros], 2); eval Reject',
        })
        .done(),
    feature('coder_typeof', 'interop')
        .unsupported('parse keeps Member call; coder.typeof runtime still open (was silent → typeof(1))')
        .pure()
        .gap('coder.typeof', 'coder.typeof(1)', {
            expected: '...',
            notes: 'Form Application(Member[coder, typeof], 1); eval Reject',
        })
        .done(),
    feature('matlab_lang_on', 'interop')
        .unsupported('parse keeps nested Member path; matlab.lang… runtime still open (was silent → on)')
        .pure()
        .gap('matlab.lang.on', 'matlab.lang.OnOffSwitchState.on', {
            expected: '...',
            notes: 'Form Member chain retained; eval Reject',
        })
        .done(),
];
