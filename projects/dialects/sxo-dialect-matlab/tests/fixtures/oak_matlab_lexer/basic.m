% MATLAB Test File - Comprehensive Syntax Coverage
% This file tests various MATLAB syntax elements for lexer testing

%% Script Header and Documentation
% MATLAB Comprehensive Test Script
% Author: Test Suite
% Date: 2024
% Purpose: Test lexer with various MATLAB syntax elements

%% Clear workspace and command window
clear all; %#ok<CLALL>
close all;
clc;

%% Constants and Basic Variables
% Scalar variables
a = 5;
b = 3.14159;
c = 2.5e-3;
d = 1.23e+10;
e = -42;
f = inf;
g = -inf;
h = NaN;

% Complex numbers
z1 = 3 + 4i;
z2 = 2 + 3j;
z3 = complex(1, 2);

% Logical variables
flag1 = true;
flag2 = false;
flag3 = logical(1);

% String and character arrays
str1 = 'Hello World';
str2 = "Modern string";
char_array = ['a', 'b', 'c'];

%% Arrays and Matrices
% Row vectors
row_vec = [1, 2, 3, 4, 5];
row_vec2 = [1 2 3 4 5];

% Column vectors
col_vec = [1; 2; 3; 4; 5];

% Matrices
matrix2x3 = [1, 2, 3; 4, 5, 6];
matrix3x3 = [1, 2, 3; 4, 5, 6; 7, 8, 9];

% Special matrices
zeros_mat = zeros(3, 4);
ones_mat = ones(2, 5);
eye_mat = eye(4);
rand_mat = rand(3, 3);
randn_mat = randn(2, 4);

% Array indexing and slicing
element = matrix3x3(2, 3);
row_slice = matrix3x3(2, :);
col_slice = matrix3x3(:, 1);
submatrix = matrix3x3(1:2, 2:3);

% Linear indexing
linear_element = matrix3x3(5);

%% Cell Arrays
% Cell array creation
cell_array = {1, 'hello', [1, 2, 3], true};
mixed_cell = {'string', 42, rand(2,2), {1, 2}};

% Cell array indexing
cell_content = cell_array{1};
cell_reference = cell_array(1);

%% Structures
% Structure creation
person.name = 'John Doe';
person.age = 30;
person.height = 5.9;
person.married = true;

% Nested structures
company.name = 'TechCorp';
company.employees(1).name = 'Alice';
company.employees(1).position = 'Engineer';
company.employees(2).name = 'Bob';
company.employees(2).position = 'Manager';

% Dynamic field names
field_name = 'dynamic_field';
person.(field_name) = 'dynamic value';

%% Arithmetic Operations
% Basic arithmetic
sum_result = a + b;
diff_result = a - b;
prod_result = a * b;
div_result = a / b;
power_result = a ^ b;
mod_result = mod(a, 3);

% Element-wise operations
vec1 = [1, 2, 3];
vec2 = [4, 5, 6];
elem_mult = vec1 .* vec2;
elem_div = vec1 ./ vec2;
elem_power = vec1 .^ vec2;

% Matrix operations
mat1 = [1, 2; 3, 4];
mat2 = [5, 6; 7, 8];
mat_mult = mat1 * mat2;
mat_transpose = mat1';
mat_inverse = inv(mat1);
mat_determinant = det(mat1);

%% Logical Operations
% Logical operators
and_result = true && false;
or_result = true || false;
not_result = ~true;

% Element-wise logical operations
logical_vec1 = [true, false, true];
logical_vec2 = [false, true, true];
elem_and = logical_vec1 & logical_vec2;
elem_or = logical_vec1 | logical_vec2;
elem_not = ~logical_vec1;

% Comparison operations
comp1 = a > b;
comp2 = a < b;
comp3 = a >= b;
comp4 = a <= b;
comp5 = a == b;
comp6 = a ~= b;

%% Control Flow Structures

% If-else statements
if a > b
    disp('a is greater than b');
elseif a < b
    disp('a is less than b');
else
    disp('a equals b');
end

% Switch statement
switch a
    case 1
        disp('a is 1');
    case {2, 3, 4}
        disp('a is 2, 3, or 4');
    case 5
        disp('a is 5');
    otherwise
        disp('a is something else');
end

% For loops
for i = 1:10
    fprintf('Iteration %d\n', i);
end

for i = 1:2:10  % Step size of 2
    fprintf('Odd iteration %d\n', i);
end

for i = [1, 3, 5, 7, 9]
    fprintf('Array iteration %d\n', i);
end

% Nested for loops
for i = 1:3
    for j = 1:3
        fprintf('i=%d, j=%d\n', i, j);
    end
end

% While loop
counter = 1;
while counter <= 5
    fprintf('While counter: %d\n', counter);
    counter = counter + 1;
end

% Break and continue
for i = 1:10
    if i == 3
        continue;
    end
    if i == 8
        break;
    end
    fprintf('Loop value: %d\n', i);
end

%% Function Definitions

% Simple function
function result = add_numbers(x, y)
    result = x + y;
end

% Function with multiple outputs
function [sum_val, diff_val, prod_val] = basic_operations(x, y)
    sum_val = x + y;
    diff_val = x - y;
    prod_val = x * y;
end

% Function with variable arguments
function result = sum_all(varargin)
    result = 0;
    for i = 1:nargin
        result = result + varargin{i};
    end
end

% Function with default arguments
function result = power_function(base, exponent)
    if nargin < 2
        exponent = 2;  % Default exponent
    end
    result = base ^ exponent;
end

% Anonymous functions
square = @(x) x.^2;
add_func = @(x, y) x + y;
complex_func = @(x) sin(x) + cos(x);

%% Mathematical Functions
% Trigonometric functions
angle = pi/4;
sin_val = sin(angle);
cos_val = cos(angle);
tan_val = tan(angle);
asin_val = asin(0.5);
acos_val = acos(0.5);
atan_val = atan(1);

% Hyperbolic functions
sinh_val = sinh(1);
cosh_val = cosh(1);
tanh_val = tanh(1);

% Exponential and logarithmic functions
exp_val = exp(1);
log_val = log(10);
log10_val = log10(100);
log2_val = log2(8);

% Power and root functions
sqrt_val = sqrt(16);
power_val = power(2, 3);
nthroot_val = nthroot(27, 3);

% Rounding functions
ceil_val = ceil(3.7);
floor_val = floor(3.7);
round_val = round(3.7);
fix_val = fix(-3.7);

%% String Operations
% String concatenation
str_concat1 = [str1, ' ', str2];
str_concat2 = strcat(str1, str2);

% String comparison
str_equal = strcmp(str1, str2);
str_equal_ignore_case = strcmpi(str1, str2);

% String searching
str_find = strfind(str1, 'World');
str_contains = contains(str1, 'Hello');

% String replacement
str_replace = strrep(str1, 'World', 'MATLAB');

% String conversion
num_str = num2str(42);
str_num = str2num('42');

%% File I/O Operations
% File writing
filename = 'test_output.txt';
fid = fopen(filename, 'w');
if fid ~= -1
    fprintf(fid, 'Hello MATLAB\n');
    fprintf(fid, 'Number: %d\n', 42);
    fprintf(fid, 'Float: %.2f\n', 3.14159);
    fclose(fid);
end

% File reading
if exist(filename, 'file')
    fid = fopen(filename, 'r');
    if fid ~= -1
        file_content = fread(fid, '*char')';
        fclose(fid);
        disp('File content:');
        disp(file_content);
    end
end

% CSV operations
data_matrix = [1, 2, 3; 4, 5, 6; 7, 8, 9];
csvwrite('data.csv', data_matrix);
loaded_data = csvread('data.csv');

%% Plotting and Graphics
% Basic 2D plot
x = linspace(0, 2*pi, 100);
y1 = sin(x);
y2 = cos(x);

figure(1);
plot(x, y1, 'r-', x, y2, 'b--');
xlabel('x');
ylabel('y');
title('Sine and Cosine Functions');
legend('sin(x)', 'cos(x)');
grid on;

% Subplot
figure(2);
subplot(2, 2, 1);
plot(x, y1);
title('Sine');

subplot(2, 2, 2);
plot(x, y2);
title('Cosine');

subplot(2, 2, 3);
plot(x, tan(x));
title('Tangent');
ylim([-5, 5]);

subplot(2, 2, 4);
plot(x, exp(-x/2) .* sin(x));
title('Damped Sine');

% 3D plotting
[X, Y] = meshgrid(-2:0.1:2, -2:0.1:2);
Z = X.^2 + Y.^2;

figure(3);
surf(X, Y, Z);
xlabel('X');
ylabel('Y');
zlabel('Z');
title('3D Surface Plot');
colorbar;

% Histogram
random_data = randn(1000, 1);
figure(4);
histogram(random_data, 30);
title('Histogram of Random Data');
xlabel('Value');
ylabel('Frequency');

%% Signal Processing
% Generate signals
fs = 1000;  % Sampling frequency
t = 0:1/fs:1-1/fs;  % Time vector
f1 = 50;  % Frequency 1
f2 = 120; % Frequency 2

signal = sin(2*pi*f1*t) + 0.5*sin(2*pi*f2*t) + 0.1*randn(size(t));

% FFT analysis
Y = fft(signal);
P2 = abs(Y/length(signal));
P1 = P2(1:length(signal)/2+1);
P1(2:end-1) = 2*P1(2:end-1);
f = fs*(0:(length(signal)/2))/length(signal);

figure(5);
subplot(2, 1, 1);
plot(t(1:100), signal(1:100));
title('Time Domain Signal');
xlabel('Time (s)');
ylabel('Amplitude');

subplot(2, 1, 2);
plot(f, P1);
title('Frequency Domain');
xlabel('Frequency (Hz)');
ylabel('Magnitude');

%% Linear Algebra Operations
% Matrix operations
A = [1, 2, 3; 4, 5, 6; 7, 8, 10];
b = [1; 2; 3];

% Solve linear system Ax = b
x_solution = A \ b;

% Eigenvalues and eigenvectors
[eigenvectors, eigenvalues] = eig(A);

% Singular Value Decomposition
[U, S, V] = svd(A);

% QR decomposition
[Q, R] = qr(A);

% LU decomposition
[L, U_lu, P] = lu(A);

%% Statistics and Probability
% Basic statistics
data = randn(100, 1);
mean_val = mean(data);
median_val = median(data);
std_val = std(data);
var_val = var(data);
min_val = min(data);
max_val = max(data);

% Correlation and covariance
data2 = randn(100, 1);
correlation = corrcoef(data, data2);
covariance = cov(data, data2);

% Probability distributions
normal_samples = normrnd(0, 1, [100, 1]);
uniform_samples = unifrnd(0, 1, [100, 1]);
exponential_samples = exprnd(1, [100, 1]);

%% Object-Oriented Programming
% Class definition (would typically be in separate file)
classdef SimpleClass < handle
    properties
        value
        name
    end
    
    properties (Access = private)
        private_data
    end
    
    methods
        function obj = SimpleClass(initial_value, initial_name)
            if nargin > 0
                obj.value = initial_value;
            end
            if nargin > 1
                obj.name = initial_name;
            end
        end
        
        function result = getValue(obj)
            result = obj.value;
        end
        
        function setValue(obj, new_value)
            obj.value = new_value;
        end
        
        function display_info(obj)
            fprintf('Name: %s, Value: %g\n', obj.name, obj.value);
        end
    end
    
    methods (Static)
        function result = static_method(x, y)
            result = x + y;
        end
    end
end

%% Error Handling
% Try-catch blocks
try
    risky_operation = 1 / 0;
    disp('This should not print');
catch ME
    fprintf('Error caught: %s\n', ME.message);
    fprintf('Error identifier: %s\n', ME.identifier);
end

% Custom error throwing
function validate_input(x)
    if x < 0
        error('Input must be non-negative');
    end
    if ~isnumeric(x)
        error('Input must be numeric');
    end
end

%% Advanced Features

% Parallel computing (if Parallel Computing Toolbox available)
if license('test', 'Distrib_Computing_Toolbox')
    parfor i = 1:10
        parallel_result(i) = expensive_computation(i);
    end
end

function result = expensive_computation(n)
    result = sum(1:n);
    pause(0.1);  % Simulate computation time
end

% GPU computing (if Parallel Computing Toolbox available)
if license('test', 'Distrib_Computing_Toolbox') && gpuDeviceCount > 0
    gpu_array = gpuArray([1, 2, 3, 4, 5]);
    gpu_result = gpu_array .^ 2;
    cpu_result = gather(gpu_result);
end

% Symbolic math (if Symbolic Math Toolbox available)
if license('test', 'Symbolic_Toolbox')
    syms x y z;
    symbolic_expr = x^2 + y^2 + z^2;
    symbolic_diff = diff(symbolic_expr, x);
    symbolic_int = int(sin(x), x);
end

%% Performance Timing
% Timing code execution
tic;
large_matrix = rand(1000, 1000);
large_result = large_matrix * large_matrix;
elapsed_time = toc;
fprintf('Matrix multiplication took %.4f seconds\n', elapsed_time);

% Profiling
profile on;
for i = 1:1000
    temp_result = sin(i) + cos(i);
end
profile off;
profile viewer;

%% Memory Management
% Check memory usage
memory_info = memory;
fprintf('Available memory: %.2f GB\n', memory_info.MemAvailableAllArrays / 1e9);

% Clear specific variables
clear temp_result large_matrix large_result;

% Pack memory (remove fragmentation)
pack;

%% System Information
% MATLAB version and system info
matlab_version = version;
computer_type = computer;
java_version = version('-java');

fprintf('MATLAB Version: %s\n', matlab_version);
fprintf('Computer Type: %s\n', computer_type);
fprintf('Java Version: %s\n', java_version);

%% Final Cleanup
% Close all figures
close all;

% Display completion message
fprintf('\n=== MATLAB Comprehensive Test Completed ===\n');
fprintf('All syntax elements have been tested.\n');
fprintf('Check workspace for created variables and results.\n');

%% End of script