iptables -F 
iptables -t nat -F
iptables -A FORWARD -i anp0 -s 11.45.14.0/24 -j ACCEPT 
iptables -A FORWARD -i wlp47s0 -d 11.45.14.0/24 -j ACCEPT 
iptables -t nat -A POSTROUTING -o wlp47s0 -j MASQUERADE
