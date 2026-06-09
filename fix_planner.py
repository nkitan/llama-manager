import sys

path = 'src-ui/src/tabs/remaining.rs'
with open(path, 'r') as f:
    lines = f.readlines()

print(f"Total lines before: {len(lines)}")

# Delete lines 1722-1932 (1-indexed), which is indices 1721-1931 in 0-indexed
# Keep lines[:1720] and lines[1932:]
new_lines = lines[:1720] + lines[1932:]

print(f"Total lines after: {len(new_lines)}")
print("Lines around 1720:")
for i, l in enumerate(new_lines[1718:1724], start=1719):
    print(f"  {i}: {l}", end='')

with open(path, 'w') as f:
    f.writelines(new_lines)
print("Done!")
