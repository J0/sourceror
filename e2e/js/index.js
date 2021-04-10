import Harness from './harness';

const harness = new Harness;
const code = `
function factorial(n) { 
    function facloop(n, acc) {
        return n === 1 
               ? acc
               : facloop(n - 1, acc * n);
    }
    return facloop(n, 1); 
}
factorial(10);
`;
harness.runCode(code, eval(code));
