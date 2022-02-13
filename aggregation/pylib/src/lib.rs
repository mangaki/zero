
use std::collections::BTreeMap;
use std::num::Wrapping;
use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::exceptions;

use aggregation::sodium_bindings::*;
use aggregation::types::*;
use aggregation::user::*;
use aggregation::server::*;

#[pyclass]
#[derive(Clone)]
struct PublicKeysWrapper(Arc<BTreeMap<usize, SignPublicKey>>);

#[pymethods]
impl PublicKeysWrapper {
    #[new]
    pub fn new() -> Self {
        PublicKeysWrapper(Arc::new(BTreeMap::new()))
    }

    pub fn insert(mut self_: PyRefMut<Self>, u: usize, pk: SignPublicKey) -> PyResult<()> {
        match Arc::get_mut(&mut self_.0) {
            Some(x) => { x.insert(u, pk); Ok(()) },
            None => Err(PyErr::new::<exceptions::PyTypeError, _>(
                    format!("Expected {} bytes, received {}.", SIGN_PUBLIC_KEY_BYTES, pk.len())))

        }
    }
}

#[pyclass]
struct UserWrapper(User);

#[pymethods]
impl UserWrapper {
    #[new]
    pub fn new(
        id: usize,
        threshold: usize,
        sign_pk: SignPublicKey,
        sign_sk: SignSecretKey,
        grad: Vec<i64>,
        others_sign_pks: PublicKeysWrapper,
    ) -> Self {
        UserWrapper(User::new(
            id, threshold, sign_pk, sign_sk,
            grad.into_iter().map(Wrapping).collect(),
            others_sign_pks.0
        ))
    }

    pub fn serialize_state(self_: PyRef<Self>) -> PyResult<String> {
        match self_.0.serialize_state() {
            Ok(s) => Ok(s),
            Err(_) => Err(PyErr::new::<exceptions::PyIOError, _>(()))
        }
    }

    pub fn recover_state(mut self_: PyRefMut<Self>, s: &str) -> PyResult<()> {
        match self_.0.recover_state(s) {
            Ok(()) => Ok(()),
            Err(_) => Err(PyErr::new::<exceptions::PyIOError, _>(()))
        }
    }

    pub fn round<'a>(mut self_: PyRefMut<Self>, py: Python<'a>, input: &[u8]) -> PyResult<&'a PyBytes> {
        match self_.0.round_serialized(input) {
            Ok(output) => Ok(PyBytes::new(py, &output)),
            Err(_) => Err(PyErr::new::<exceptions::PyIOError, _>(()))
        }
    }
}

#[pyclass]
struct ServerOutputWrapper(ServerOutputSerialized);

impl ServerOutputWrapper {
    pub fn new(wrapped: ServerOutputSerialized) -> Self {
        ServerOutputWrapper(wrapped)
    }
}

#[pymethods]
impl ServerOutputWrapper {
    pub fn is_messages(self_: PyRef<Self>) -> bool {
        match &self_.0 {
            ServerOutputSerialized::Messages(_) => true,
            _ => false,
        }
    }
    
    pub fn is_gradient(self_: PyRef<Self>) -> bool {
        match &self_.0 {
            ServerOutputSerialized::Gradient(_) => true,
            _ => false,
        }
    }

    pub fn get_messages<'a>(self_: PyRef<Self>, py: Python<'a>) -> PyResult<BTreeMap<usize, &'a PyBytes>> {
        match &self_.0 {
            ServerOutputSerialized::Messages(output) =>
                Ok(output.into_iter().map(|(k, v)| (*k, PyBytes::new(py, &v))).collect()),
            _ => Err(PyErr::new::<exceptions::PyTypeError, _>(()))
        }
    }

    pub fn get_gradient(self_: PyRef<Self>) -> PyResult<Vec<i64>> {
        match &self_.0 {
            ServerOutputSerialized::Gradient(v) => Ok(v.iter().map(|Wrapping(i)| *i).collect()),
            _ => Err(PyErr::new::<exceptions::PyTypeError, _>(()))
        }
    }
}

#[pyclass]
struct ServerWrapper {
    wrapped: Server,
}

#[pymethods]
impl ServerWrapper {
    #[new]
    pub fn new(threshold: usize, grad_len: usize) -> Self {
        ServerWrapper { wrapped: Server::new(threshold, grad_len) }
    }

    pub fn serialize_state(self_: PyRef<Self>) -> PyResult<String> {
        match self_.wrapped.serialize_state() {
            Ok(s) => Ok(s),
            Err(_) => Err(PyErr::new::<exceptions::PyIOError, _>(()))
        }
    }

    pub fn recover_state(mut self_: PyRefMut<Self>, s: &str) -> PyResult<()> {
        match self_.wrapped.recover_state(s) {
            Ok(()) => Ok(()),
            Err(_) => Err(PyErr::new::<exceptions::PyIOError, _>(()))
        }
    }

    pub fn recv<'a>(mut self_: PyRefMut<Self>, id: usize, input: &[u8]) -> PyResult<()> {
        match self_.wrapped.recv_serialized(id, input) {
            Ok(()) => Ok(()),
            Err(_) => Err(PyErr::new::<exceptions::PyIOError, _>(()))
        }
    }

    pub fn round<'a>(mut self_: PyRefMut<Self>) -> PyResult<ServerOutputWrapper> {
        match self_.wrapped.round_serialized() {
            Ok(output) => Ok(ServerOutputWrapper::new(output)),
            Err(_) => Err(PyErr::new::<exceptions::PyIOError, _>(()))
        }
    }
}

#[pyfunction]
fn round0_msg<'a>(py: Python<'a>) -> &'a PyBytes {
    PyBytes::new(py, &bincode::serialize(&UserInput::Round0()).unwrap())
}

#[pyfunction]
fn gen_keypair() -> (SignPublicKey, SignSecretKey) {
    gen_sign_keypair()
}

#[pymodule]
fn aggregation(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(gen_keypair, m)?)?;
    m.add_function(wrap_pyfunction!(round0_msg, m)?)?;
    m.add_class::<PublicKeysWrapper>()?;
    m.add_class::<UserWrapper>()?;
    m.add_class::<ServerWrapper>()?;
    Ok(())
}

