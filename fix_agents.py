path = 'src-ui/src/tabs/remaining.rs'
with open(path, 'r') as f:
    lines = f.readlines()

print(f"Total lines before: {len(lines)}")

# Delete orphaned old AgentsTab code from lines 1252-1523 (1-indexed)
# = indices 1251-1522 in 0-indexed
# Keep lines[:1251] and lines[1523:]
new_lines = lines[:1251] + lines[1523:]

print(f"Total lines after: {len(new_lines)}")

# Verify the join
print("Lines around 1250:")
for i, l in enumerate(new_lines[1248:1256], start=1249):
    print(f"  {i}: {l}", end='')

with open(path, 'w') as f:
    f.writelines(new_lines)
print("Done!")
