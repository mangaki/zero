
API
===

.. autosummary:
   :toctree: generated

.. module:: mangaki_zero_aggregation

.. py:class:: SignPublicKey
   Alias for bytes

.. py:class:: SignPrivateKey
   Alias for bytes

.. py:class:: PublicKeysWrapper
   .. py:method:: insert()

.. py:class:: UserWrapper

    .. py:method:: __new__(id, threshold, sign_pk, sign_sk, vec, others_sign_pks)

       :param int id:
       :param int threshold:
       :param SignPublicKey sign_pk:
       :param SignSecretKey sign_sk:
       :param list[int] vec:
       :param PublicKeysWrapper other_sign_pks:
       :rtype: UserWrapper

    .. py:method:: serialize_state()
       
       :rtype: str

    .. py:method:: recover_state(state)

       :param str state:

    .. py:method:: round(msg, input)

       :param bytes msg:
       :param list[int] input:

.. py:class:: ServerOutputWrapper

   .. py:method:: is_messages()
  
      :rtype: bool

   .. py:method:: is_vector()

      :rtype: bool

   .. py:method:: get_messages()

      :rtype: Dict[int, bytes]
      :raises IOError:
    
   .. py:method:: get_vector()

      :rtype: list[int]
      :raises IOError:

.. py:class:: ServerWrapper
    
   .. py:method:: __new__(cls, threshold: int, vec_len: int) -> 'ServerWrapper': ...

      :param int threshold:
      :param int vec_len:

   .. py:method:: serialize_state()

      :rtype: str

   .. py:method:: recover_state(state)

      :param str state:

   .. py:method:: recv(id, input)

      :param int id:
      :param list[int] input:

   .. py:method:: round()

      :rtype: ServerOutputWrapper

.. py:function:: round0_msg()

   :rtype: bytes

.. py:function:: gen_keypair()
   
   :rtype: Tuple[SignPublicKey, SignSecretKey]
