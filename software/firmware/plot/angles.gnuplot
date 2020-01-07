reset

set term qt noraise

unset key

stats 'angles.dat' using 1 name "X" nooutput

set xrange [X_max-10000:X_max]
set yrange [-pi:2*pi]
set y2range [-100:100]
set ytics ('-π' -pi, 0, 'π' pi, '2π' 2*pi)
set y2tics

plot 'angles.dat' using 1:2 with lines axes x1y1, 'angles.dat' using 1:3 with lines axes x1y1, 'angles.dat' using 1:4 with lines axes x1y1

pause 0.01
reread

