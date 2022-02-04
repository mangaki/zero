
import random

from aggregation import *

participants = 8
active = 6
threshold = 3
grad_len = 9
mask = [i < active for i in range(participants)]
random.Random(45).shuffle(mask)

sign_keys = [gen_keypair() for _ in range(participants)]

sign_pks = PublicKeysWrapper()
for (u, (pk, _)) in enumerate(sign_keys):
    sign_pks.insert(u, pk)

users = [ UserWrapper(u, threshold, pk, sk, [(u + 1 if i == u else 0) for i in range(grad_len)], sign_pks)
            for (u, (pk, sk)) in enumerate(sign_keys) ]
server = ServerWrapper(threshold, grad_len)

msgs = {i: round0_msg() for i in range(participants)}
server_output = None
round = 0

while True:
    for (i, user) in enumerate(users):
        if round < 2 or mask[i]:
            server.recv(i, user.round(msgs[i]))

    server_output = server.round()

    if(server_output.is_messages()):
        msgs = server_output.get_messages()
        round += 1
    else:
        break

print(server_output.get_gradient())

