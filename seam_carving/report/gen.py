w = 15
h = 10

print('digraph calc {')
print('  node [shape=box]')
for i in range(1, w+1):
    for j in range(1, h+1):
        print(
            f'  p{j}_{i} [label = "{j},{i}" width = 1.0 style = filled, fillcolor = bisque1, group = 1 ];')
    print()


# print('  subgraph cluster_th1 {')
# print('    label="thread1";')
for i in range(1, 6):
    for j in range(1, h+1):
        print(
            f'    p{j}_{i} [label = "{j},{i}" width = 1.0 style = filled, fillcolor = red, group = 1 ];')
    print()
# print('  }')
# print()

# print('  subgraph cluster_th2 {')
# print('    label="thread2";')
for i in range(6, 11):
    for j in range(1, h+1):
        print(
            f'    p{j}_{i} [label = "{j},{i}" width = 1.0 style = filled, fillcolor = green, group = 1 ];')
    print()
# print('  }')
# print()

# print('  subgraph cluster_th3 {')
# print('    label="thread3";')
for i in range(11, 16):
    for j in range(1, h+1):
        print(
            f'    p{j}_{i} [label = "{j},{i}" width = 1.0 style = filled, fillcolor = blue, group = 1 ];')
    print()
# print('  }')
# print()

for i in range(1, h + 1):
    print(f'  subgraph cluster_loop{i} {{')
    print(f'    label="loop{i}";')
    for j in range(1, w + 1):
        print(f'    p{i}_{j};')
    print('  }')
    print()

# for i in range(1, h+1):
#     for j in range(1 + (i-1) % 3, 6 - (i-1) % 3):
#         print(
#             f'  p{i}_{j} [label = "{i},{j}" width = 1.0 style = filled, fillcolor = red, group = 1 ];')

#     for j in range(6 + (i-1) % 3, 11 - (i-1) % 3):
#         print(
#             f'  p{i}_{j} [label = "{i},{j}" width = 1.0 style = filled, fillcolor = green, group = 1 ];')

#     for j in range(11 + (i-1) % 3, 16 - (i-1) % 3):
#         print(
#             f'  p{i}_{j} [label = "{i},{j}" width = 1.0 style = filled, fillcolor = blue, group = 1 ];')

#     for j in range(6 - (i-1) % 3, 6 + (i-1) % 3):
#         print(
#             f'  p{i}_{j} [label = "{i},{j}" width = 1.0 style = filled, fillcolor = yellow, group = 1 ];')

#     for j in range(11 - (i-1) % 3, 11 + (i-1) % 3):
#         print(
#             f'  p{i}_{j} [label = "{i},{j}" width = 1.0 style = filled, fillcolor = cyan, group = 1 ];')

#     for j in range(1, 1 + (i-1) % 3):
#         print(
#             f'  p{i}_{j} [label = "{i},{j}" width = 1.0 style = filled, fillcolor = purple, group = 1 ];')

#     for j in range(16 - (i-1) % 3, 16):
#         print(
#             f'  p{i}_{j} [label = "{i},{j}" width = 1.0 style = filled, fillcolor = purple, group = 1 ];')


# print('  subgraph cluster_loop1 {')
# print('    label="loop1";')
# for i in range(1, 4):
#     for j in range(1, w + 1):
#          print(
#             f'  p{i}_{j};')
# print('  }')
# print()

# print('  subgraph cluster_loop2 {')
# print('    label="loop2";')
# for i in range(4, 7):
#     for j in range(1, w + 1):
#          print(
#             f'  p{i}_{j};')
# print('  }')
# print()

# print('  subgraph cluster_loop3 {')
# print('    label="loop3";')
# for i in range(7, 10):
#     for j in range(1, w + 1):
#          print(
#             f'  p{i}_{j};')
# print('  }')
# print()

# print('  subgraph cluster_loop4 {')
# print('    label="loop4";')
# for i in range(10, 11):
#     for j in range(1, w + 1):
#          print(
#             f'  p{i}_{j};')
# print('  }')
# print()
# print('  { rank = same; ', end='')
# for i in range(1, w+1):
#     print(f'p1_{i}; ', end='')
# print('}')

for i in range(1, w+1):
    if i != 1:
        for j in range(1, h):
            print(f'  p{j}_{i} -> p{j+1}_{i-1};')
        print()
    for j in range(1, h):
        print(f'  p{j}_{i} -> p{j+1}_{i};')
    print()
    if i != w:
        for j in range(1, h):
            print(f'  p{j}_{i} -> p{j+1}_{i+1};')
        print()

print('}')
