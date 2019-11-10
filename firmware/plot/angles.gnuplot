reset

set term qt noraise

unset key

stats 'angles.dat' using 1 name "X" nooutput

set xrange [X_max-10000:X_max]
set yrange [-7:7]
set y2range [-100:100]
set ytics ('-2π' -2*pi, '-π' -pi, 0, 'π' pi, '2π' 2*pi)
set y2tics

plot 'angles.dat' using 1:2 with lines axes x1y1 plot 'angles.dat' using 1:2 with lines axes x1y1, 'angles.dat' using 1:3 with lines axes x1y1, 'angles.dat' using 1:4 with lines axes x1y2

pause 0.01
reread

