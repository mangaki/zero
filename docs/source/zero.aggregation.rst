zero.aggregation package: secure aggregation
============================================

.. autosummary:
   :toctree: generated

Bindings to our Rust implementation of `Practical Secure Aggregation for Privacy-Preserving Machine Learning <https://eprint.iacr.org/2017/281.pdf>`_.
We give an overview of this in our `blog post <https://research.mangaki.fr/2022/02/12/how-to-securely-share-information/>`_.

In this protocol, a central server collects vectors from a certain number of clients
and computes the sum, following a scheme that guarantees data privacy (see the paper for
more precisions, in particular on the matter of privacy).
The protocol is divided in `rounds`, where the server sends a message to each participant,
and the participants each respond with their own message.
At each round, the server awaits at list ``threshold`` messages from participants, performs a computation and
then sends new messages to each participant.
There are 5 rounds (see the paper for reference); at the final round the server outputs
not messages but the securely aggregated sum of each user's vector.

.. module:: zero.aggregation

.. py:class:: SignPublicKey

   A public key that is used internally to allow others to verify signatures.

   Generated by :py:func:`gen_keypair`.

   Alias for bytes.

.. py:class:: SignSecretKey

   A private key that is used internally to sign messages.

   Generated by :py:func:`gen_keypair`.

   Alias for bytes.

.. py:class:: PublicKeysWrapper

   An interface to build a dictionnary that maps user ids to :py:class:`SignPublicKey`'s.

   .. py:method:: insert(u, pk)

      Registers a public key for a user id.

      :param int u:
      :param SignPublicKey pk:

.. py:class:: UserWrapper

   Wraps the internal ``User`` object.

   This handles all the client side of the protocol.
   The session parameters and the data to share are given at the creation of the ``UserWrapper``,
   then it suffices to respond to the server's messages.

    .. py:method:: __new__(id, threshold, sign_pk, sign_sk, vec, others_sign_pks)

       :param int id: The identifier of this user
       :param int threshold: The minimum number of user that must participate for the aggregation to continue
       :param SignPublicKey sign_pk: This user's public key
       :param SignSecretKey sign_sk: This user's private key
       :param list[int] vec: The vector to send -- kept confidential by the protocol (see the paper for more information)
       :param PublicKeysWrapper other_sign_pks: The public keys of all the users who participate in the aggregation
       :rtype: UserWrapper

    .. py:method:: round(input)

       This batteries-included function handles the client side of the secure aggregation protocol.
       It expects a message from the server and outputs a new message that must be sent to the server.

       :param bytes input: The message that the server sent
       :rtype: bytes
       :raises IOError: If the input is invalid, or a tamper is detected -- the client is then put in an error state

    .. py:method:: serialize_state()

       Writes the state of this object to a string.
       The data is formatted in JSON (which can be helpful for storages that optimize JSON data,
       like PostgreSQL). However the exact contents of the serialized object
       are not specified.

       The output contains secrets so `do not render the output of this function public`.
       
       :rtype: str

    .. py:method:: recover_state(state)

       Recovers the state that has been serialized by :py:meth:`serialize_state`.

       :param str state: The string that was given by a previous call to :py:meth:`serialize_state`

.. py:class:: ServerOutputWrapper

   The server can output two sorts of things after a round:
   either messages for each participant, or, after the last round, the result vector.

   This class represents these outputs, and is essentially an ad-hoc implementation of a sum type.

   .. py:method:: is_messages()

      Did the server output messages?
  
      :rtype: bool

   .. py:method:: is_vector()

      Did the server output a vector?

      :rtype: bool

   .. py:method:: get_messages()

      If the server did output messages, returns a dictionary that maps each user identifier
      to the message that must be sent to that user.

      :rtype: Dict[int, bytes]
      :raises IOError: If the server actually outputted a vector.
    
   .. py:method:: get_vector()

      If the server did output a vector, returns that vector.

      :rtype: list[int]
      :raises IOError: If the server actually outputted messages.

.. py:class:: ServerWrapper
    
   .. py:method:: __new__(cls, threshold: int, vec_len: int) -> 'ServerWrapper': ...

      :param int threshold: The minimum number of user that must participate for the aggregation to continue
      :param int vec_len: The dimension of the vectors that will be sent by the users

   .. py:method:: recv(id, input)

      Signals that a message was sent by a user.

      Must be called before :py:meth:`round`.

      :param int id: The user's identifier
      :param bytes input: The message
      :raises IOError: If the input is invalid -- but the server isn't in an error state after that

   .. py:method:: round()

      Computes, based on the messages that were received for this round through :py:meth:`recv`,
      the result of the current round (either new messages for the users, or the computed sum).

      :rtype: ServerOutputWrapper
      :raises IOError: If less than ``threshold`` messages have been received, or if the received data is incoherent -- the server is put in an error state after that

   .. py:method:: serialize_state()

       Writes the state of this object to a string.
       The data is formatted in JSON (which can be helpful for storages that optimize JSON data,
       like PostgreSQL). However the exact contents of the serialized object
       are not specified.

       As opposed to :py:meth:`UserWrapper.serialize_state`, the output doesn't contain secrets,
       so it doesn't need to be specially protected.

      :rtype: str

   .. py:method:: recover_state(state)

       Recovers the state that has been serialized by :py:meth:`serialize_state`.

      :param str state: The string that was given by a previous call to :py:meth:`serialize_state`

.. py:function:: round0_msg()

   Returns the message that must be given to each user for the first round.

   :rtype: bytes

.. py:function:: gen_keypair()

   Generates a pair of keys that will be used internally to verify that the participants can be trusted.

   See the paper for more information.
   
   :rtype: Tuple[SignPublicKey, SignSecretKey]
