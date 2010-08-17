import { feature } from '@sxo/harness';

export const formsFeatures = [
    feature('InputForm', 'forms')
        .unsupported('HoldAllComplete Form kept; no InputForm pretty-print runtime')
        .pure()
        .gap('inputform.plus', 'InputForm[1 + 1]', {
            expected: 'InputForm[1 + 1]',
            notes: 'must stay InputForm[1 + 1], not InputForm[2]',
        })
        .done(),
    feature('FullForm', 'forms').unsupported().pure().gap('fullform.plus', 'FullForm[1 + x]', { expected: 'Plus[1, x]' }).done(),
    feature('TeXForm', 'forms').unsupported().pure().gap('texform.x2', 'TeXForm[x^2]', { expected: 'x^2' }).done(),
];
