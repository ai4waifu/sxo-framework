import { feature } from '@sxo/harness';

export const plotFeatures = [
    feature('plot', 'plot')
        .partial(
            'SVG→PNG visual: curve+L-axes readable; missing tick labels, no boxed frame, not MATLAB default blue. Negative a/b via unary-minus fold; 2-arg plot(f,[a,b]) gap',
        )
        .effectful()
        .plot('plot.square', 'plot(x^2, x, 0, 1)', { expected: '<svg' })
        .plot('plot.sin', 'plot(sin(x), x, 0, 6)', { expected: '<svg' })
        .gap('plot.neg_domain', 'plot(x^2, x, -1, 1)', { expected: '<svg', notes: 'currently not a supported 1-D plot form' })
        .gap('plot.range_vec', 'plot(sin(x), [-pi, pi])', { expected: '<svg', notes: 'surface sugar for domain vector' })
        .done(),
    feature('mesh', 'plot').unsupported().effectful().gap('mesh.peaks', 'mesh(peaks)', { expected: '<svg' }).done(),
    feature('surf', 'plot').unsupported().effectful().gap('surf.peaks', 'surf(peaks)', { expected: '<svg' }).done(),
    feature('contour', 'plot').unsupported().effectful().gap('contour.peaks', 'contour(peaks)', { expected: '<svg' }).done(),
    feature('figure', 'plot').unsupported().effectful().gap('figure.basic', 'figure', { expected: '...' }).done(),
    feature('hold_on', 'plot')
        .unsupported('SILENT WRONG: hold on → on (command keyword stripped)')
        .effectful()
        .gap('hold.on', 'hold on', { expected: '...', notes: 'currently returns on' })
        .done(),
    feature('grid_on', 'plot')
        .unsupported('SILENT WRONG: grid on → on')
        .effectful()
        .gap('grid.on', 'grid on', { expected: '...', notes: 'currently returns on' })
        .done(),
    feature('axis', 'plot')
        .unsupported('SILENT WRONG: axis equal → equal')
        .effectful()
        .gap('axis.equal', 'axis equal', { expected: '...', notes: 'currently returns equal' })
        .done(),
    feature('legend', 'plot').unsupported().effectful().gap('legend.a', "legend('a')", { expected: '...' }).done(),
    feature('subplot', 'plot').unsupported().effectful().gap('subplot.121', 'subplot(1, 2, 1)', { expected: '...' }).done(),
    feature('close_all', 'plot')
        .unsupported('SILENT WRONG: close all → all')
        .effectful()
        .gap('close.all', 'close all', { expected: '...', notes: 'currently returns all' })
        .done(),
    feature('hold_off', 'plot')
        .unsupported('SILENT WRONG: hold off → off')
        .effectful()
        .gap('hold.off', 'hold off', { expected: '...', notes: 'currently returns off' })
        .done(),
    feature('colormap', 'plot')
        .unsupported('SILENT WRONG: colormap jet → jet')
        .effectful()
        .gap('colormap.jet', 'colormap jet', { expected: '...', notes: 'currently returns jet' })
        .done(),
    feature('scatter', 'plot').unsupported().effectful().gap('scatter.basic', 'scatter([1, 2], [3, 4])', { expected: '<svg' }).done(),
    feature('bar', 'plot').unsupported().effectful().gap('bar.3', 'bar([1, 2, 3])', { expected: '<svg' }).done(),
    feature('semilogx', 'plot').unsupported().effectful().gap('semilogx.basic', 'semilogx(1:3, 1:3)', { expected: '<svg' }).done(),
    feature('loglog', 'plot').unsupported().effectful().gap('loglog.basic', 'loglog(1:3, 1:3)', { expected: '<svg' }).done(),
    feature('pie', 'plot').unsupported().effectful().gap('pie.3', 'pie([1, 2, 3])', { expected: '<svg' }).done(),
    feature('ginput', 'plot').planned().effectful().gap('ginput.basic', 'ginput', { expected: '...' }).done(),
    feature('zoom', 'plot').planned().effectful().gap('zoom.basic', 'zoom', { expected: '...' }).done(),
];
