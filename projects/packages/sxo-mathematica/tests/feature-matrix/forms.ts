import { feature } from '@sxo/harness';

export const formsFeatures = [
    feature('InputForm', 'forms')
        .unsupported('SILENT WRONG eval-before-wrap: InputForm[1+1]→InputForm[2]; Hold still forced inside')
        .pure()
        .gap('inputform.plus', 'InputForm[1 + 1]', { expected: 'InputForm[1 + 1]' })
        .done(),
    feature('FullForm', 'forms').unsupported().pure().gap('fullform.plus', 'FullForm[1 + x]', { expected: 'Plus[1, x]' }).done(),
    feature('TeXForm', 'forms').unsupported().pure().gap('texform.x2', 'TeXForm[x^2]', { expected: 'x^2' }).done(),
];
