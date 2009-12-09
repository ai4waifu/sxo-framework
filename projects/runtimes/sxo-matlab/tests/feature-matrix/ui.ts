import { feature } from '@sxo/harness';

export const uiFeatures = [
    feature('uifigure', 'ui').unsupported().effectful().gap('uifigure.basic', 'uifigure', { expected: '...' }).done(),
    feature('msgbox', 'ui').unsupported().effectful().gap('msgbox.a', "msgbox('a')", { expected: '...' }).done(),
    feature('warndlg', 'ui').unsupported().effectful().gap('warndlg.w', "warndlg('w')", { expected: '...' }).done(),
    feature('uigridlayout', 'ui').planned().effectful().gap('uigridlayout.basic', 'uigridlayout', { expected: '...' }).done(),
    feature('drawnow', 'ui').planned().effectful().gap('drawnow.basic', 'drawnow', { expected: '...' }).done(),
];
