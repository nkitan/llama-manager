path = 'src-ui/src/tabs/remaining.rs'
with open(path, 'r') as f:
    lines = f.readlines()

print(f"Total lines before: {len(lines)}")

# Delete orphaned old MonitorTab body lines 1602-1699 (1-indexed)
# = indices 1601-1698 in 0-indexed
new_lines = lines[:1601] + lines[1699:]

print(f"Total lines after: {len(new_lines)}")
print("Lines around 1600:")
for i, l in enumerate(new_lines[1598:1606], start=1599):
    print(f"  {i}: {l}", end='')

with open(path, 'w') as f:
    f.writelines(new_lines)
print("Done!")
